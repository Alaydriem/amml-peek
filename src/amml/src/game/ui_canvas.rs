use std::sync::Arc;

use amml_api::{ApiError, Color, ItemDrawFlags, ItemDrawHandle, Point, Rect as ApiRect, TextAlign, UiDraw};
use peek_core::structs::{Rect, Rgb};

use crate::traits::Canvas;

/// The canvas of one `ScreenRender` call over the `UiDraw` of the loader. Fills, the outline and items draw at once;
/// the counts and the hint are text that the loader copies and draws after the last listener. Game thread, inside the
/// listener.
pub struct UiCanvas<'a, 'f> {
    draw: &'a UiDraw<'f>,
    stale: bool,
    texts: u32,
}

impl<'a, 'f> UiCanvas<'a, 'f> {
    // the height of a line of the default font, in GUI units
    const TEXT_HEIGHT: f32 = 8.0;
    // the offset of the count below the bottom of the cell, as the vanilla stack count has
    const COUNT_OFFSET_Y: f32 = 1.0;
    const TEXT_COLOR: Color = Color::WHITE;
    const TEXT_ALPHA: f32 = 1.0;

    pub fn new(draw: &'a UiDraw<'f>) -> Self {
        Self { draw, stale: false, texts: 0 }
    }

    /// Whether an item draw met a handle of a world that was left: the items must be made again.
    pub fn stale(&self) -> bool {
        self.stale
    }

    /// The number of texts drawn so far.
    pub fn texts(&self) -> u32 {
        self.texts
    }

    fn area(rect: Rect) -> ApiRect {
        ApiRect { x: rect.x0, y: rect.y0, width: rect.width(), height: rect.height() }
    }

    fn color(color: Rgb) -> Color {
        Color { r: color.r, g: color.g, b: color.b }
    }

    fn text(&mut self, text: &str, rect: ApiRect, align: TextAlign) {
        if self.draw.text(text, rect, Self::TEXT_COLOR, Self::TEXT_ALPHA, align).is_ok() {
            self.texts += 1;
        }
    }
}

impl Canvas for UiCanvas<'_, '_> {
    type Stack = Arc<ItemDrawHandle>;

    fn fill(&mut self, area: Rect, color: Rgb, alpha: f32) {
        let _ = self.draw.fill_rect(Self::area(area), Self::color(color), alpha);
    }

    fn outline(&mut self, area: Rect, color: Rgb, alpha: f32, thickness: i32) {
        let _ = self.draw.outline_rect(Self::area(area), Self::color(color), alpha, thickness);
    }

    fn item(&mut self, stack: &Arc<ItemDrawHandle>, icon: Rect, foil: bool, alpha: f32, z_order: i32) {
        let flags = if foil { ItemDrawFlags::FOIL_ONLY } else { ItemDrawFlags::NONE };
        if let Err(ApiError::Stale) = self.draw.item(stack, Point { x: icon.x0, y: icon.y0 }, flags, alpha, z_order) {
            self.stale = true;
        }
    }

    fn count(&mut self, text: &str, cell: Rect) {
        let bottom = cell.y1 + Self::COUNT_OFFSET_Y;
        let rect =
            ApiRect { x: cell.x0, y: bottom - Self::TEXT_HEIGHT, width: cell.width(), height: Self::TEXT_HEIGHT };
        self.text(text, rect, TextAlign::Right);
    }

    fn label(&mut self, text: &str, area: Rect) {
        let rect = ApiRect { x: area.x0, y: area.y0, width: area.width(), height: Self::TEXT_HEIGHT };
        self.text(text, rect, TextAlign::Left);
    }
}
