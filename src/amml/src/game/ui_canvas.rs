use std::ffi::c_void;

use amml_game::{
    BaseActorRenderContext, GameEntries, OwnedItemStack, RectangleAreaLayout, TextMeasureDataLayout, UiColorLayout,
    UiRenderContext,
};
use peek_core::structs::{Rect, Rgb};

use crate::game::FontScope;
use crate::traits::Canvas;

// what drawing items needs, made on the first item of a frame
struct ItemDrawing {
    context: BaseActorRenderContext<'static>,
    // the local player, or null; the animation frame of a clock or compass depends on it
    holder: *mut c_void,
}

/// The canvas of one `ScreenView::render` call over the game's UI render context. Fills, the outline and items
/// draw at once; the counts are queued and drawn at `flush_text`. The font handle lives from the first count to the
/// flush. Client main thread, inside the render call.
pub struct UiCanvas<'a> {
    ui: &'a UiRenderContext<'static>,
    entries: &'static GameEntries,
    items: Option<ItemDrawing>,
    items_tried: bool,
    font: Option<FontScope>,
    font_tried: bool,
    queued: bool,
}

impl<'a> UiCanvas<'a> {
    // the height of a line of the default font at size 1, in GUI units
    const TEXT_HEIGHT: f32 = 8.0;
    // the offset of the count below the bottom of the cell, as the vanilla stack count has
    const COUNT_OFFSET_Y: f32 = 1.0;
    const FONT_SIZE: f32 = 1.0;
    const LEFT_ALIGNED: u32 = 0;
    // the delta of the text flush; the counts do not animate
    const FLUSH_DELTA: f32 = 0.0;

    pub fn new(ui: &'a UiRenderContext<'static>, entries: &'static GameEntries) -> Self {
        Self { ui, entries, items: None, items_tried: false, font: None, font_tried: false, queued: false }
    }

    fn area(rect: Rect) -> RectangleAreaLayout {
        RectangleAreaLayout::new(rect.x0, rect.x1, rect.y0, rect.y1)
    }

    fn color(color: Rgb) -> UiColorLayout {
        UiColorLayout::rgb(color.r, color.g, color.b)
    }

    // the item drawing state; made once per frame, `None` when the game cannot give it
    fn drawing(&mut self) -> Option<&ItemDrawing> {
        if !self.items_tried {
            self.items_tried = true;
            self.items = BaseActorRenderContext::for_ui(self.ui).map(|context| ItemDrawing {
                context,
                holder: self
                    .ui
                    .client()
                    .and_then(|client| client.local_player())
                    .map_or(std::ptr::null_mut(), |player| player.as_ptr()),
            });
        }
        self.items.as_ref()
    }

    // opens the font handle on the first count of the frame; false when the font is not available
    #[allow(unsafe_code)]
    fn open_font(&mut self) -> bool {
        if !self.font_tried {
            self.font_tried = true;
            if let Some(client) = self.ui.client() {
                // SAFETY: the render call's client is a live ClientInstance of the build `entries` was resolved for.
                self.font = unsafe { FontScope::open(self.entries, client.as_ptr()) };
            }
        }
        self.font.is_some()
    }
}

impl Canvas for UiCanvas<'_> {
    type Stack = OwnedItemStack<'static>;

    fn fill(&mut self, area: Rect, color: Rgb, alpha: f32) {
        let _ = self.ui.fill_rect(Self::area(area), Self::color(color), alpha);
    }

    fn outline(&mut self, area: Rect, color: Rgb, alpha: f32, thickness: i32) {
        let _ = self.ui.draw_rect(Self::area(area), Self::color(color), alpha, thickness);
    }

    #[allow(unsafe_code)]
    fn item(&mut self, stack: &OwnedItemStack<'static>, icon: Rect, foil: bool, alpha: f32, z_order: i32) {
        let Some(drawing) = self.drawing() else { return };
        let Some(renderer) = drawing.context.item_renderer() else { return };
        let base = stack.as_base();
        // SAFETY: `holder` is null or the live local player.
        let frame = unsafe { base.animation_frame(drawing.holder) }.unwrap_or(0);
        let _ = renderer.render_gui_item(&drawing.context, &base, frame, icon.x0, icon.y0, foil, alpha, 1.0, z_order);
    }

    fn count(&mut self, text: &str, cell: Rect) {
        let Some(width) = self.text_width(text) else { return };
        let bottom = cell.y1 + Self::COUNT_OFFSET_Y;
        self.queue(text, RectangleAreaLayout::new(cell.x1 - width, cell.x1, bottom - Self::TEXT_HEIGHT, bottom));
    }

    fn label(&mut self, text: &str, area: Rect) {
        let Some(width) = self.text_width(text) else { return };
        self.queue(text, RectangleAreaLayout::new(area.x0, area.x0 + width, area.y0, area.y0 + Self::TEXT_HEIGHT));
    }

    fn flush_text(&mut self) {
        if self.queued {
            let _ = self.ui.flush_text(Self::FLUSH_DELTA);
            self.queued = false;
        }
        self.font = None;
    }
}

impl UiCanvas<'_> {
    // the width of `text` in GUI units; `None` when the font is not available
    fn text_width(&mut self, text: &str) -> Option<f32> {
        if !self.open_font() {
            return None;
        }
        let font = self.font.as_ref()?;
        // a font that cannot measure gives a width of 6 units a character, which is the default font at size 1
        Some(font.line_width(text).map_or(text.len() as f32 * 6.0, |width| width as f32))
    }

    #[allow(unsafe_code)]
    fn queue(&mut self, text: &str, area: RectangleAreaLayout) {
        let Some(font_ptr) = self.font.as_ref().and_then(FontScope::font) else { return };
        // SAFETY: the font lives while `self.font` does, which is until `flush_text`; the caller keeps `text` alive
        // and unmoved until then.
        let queued = unsafe {
            self.ui.draw_text(
                font_ptr,
                area,
                text,
                UiColorLayout::rgb(1.0, 1.0, 1.0),
                1.0,
                Self::LEFT_ALIGNED,
                TextMeasureDataLayout::new(Self::FONT_SIZE, true),
            )
        };
        self.queued |= queued.is_some();
    }
}

impl Drop for UiCanvas<'_> {
    fn drop(&mut self) {
        // text queued and never flushed would be read from a font handle that is gone
        if self.queued {
            let _ = self.ui.flush_text(Self::FLUSH_DELTA);
        }
    }
}
