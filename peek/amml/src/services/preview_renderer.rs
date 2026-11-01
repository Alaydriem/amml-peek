use peek_core::services::FillBar;
use peek_core::structs::{PreviewLayout, Rect, ShulkerGrid};

use crate::services::PreviewCache;
use crate::traits::Canvas;
use crate::types::{PanelStyle, PreviewItem};

/// Draws the panel through a canvas: the frame and the cells, the items, the durability and fill bars, then the
/// counts and the hint.
pub struct PreviewRenderer;

impl PreviewRenderer {
    const SLOTS: usize = ShulkerGrid::COLUMNS * ShulkerGrid::ROWS;

    /// Draws all 27 cells whether or not the box holds anything, so an empty box shows its grid; the strip cells
    /// follow when the layout has a strip. The bundle in box slot `selected` is outlined. The counts and the hint
    /// are queued from the cache and from `hint`, which the caller keeps unchanged until this returns, and flushed
    /// once at the end.
    pub fn draw<C: Canvas>(
        layout: &PreviewLayout,
        cache: &PreviewCache<C::Stack>,
        selected: Option<u8>,
        hint: Option<&str>,
        canvas: &mut C,
    ) {
        canvas.fill(layout.frame(), PanelStyle::FRAME, PanelStyle::FRAME_ALPHA);
        for slot in 0..Self::SLOTS {
            canvas.fill(layout.icon(slot), PanelStyle::CELL, PanelStyle::CELL_ALPHA);
        }
        for position in 0..layout.strip_cells() {
            let Some(icon) = layout.strip_icon(position) else { continue };
            canvas.fill(icon, PanelStyle::CELL, PanelStyle::CELL_ALPHA);
        }
        canvas.outline(layout.frame(), PanelStyle::BORDER, PanelStyle::BORDER_ALPHA, PanelStyle::BORDER_THICKNESS);
        if let Some(slot) = selected {
            canvas.outline(
                layout.cell(usize::from(slot)),
                PanelStyle::OUTLINE,
                PanelStyle::OUTLINE_ALPHA,
                PanelStyle::OUTLINE_THICKNESS,
            );
        }
        for slot in 0..Self::SLOTS {
            if let Some(item) = cache.item(slot) {
                Self::item(item, layout.icon(slot), canvas);
            }
        }
        for position in 0..layout.strip_cells() {
            if let (Some(item), Some(icon)) = (cache.strip_item(position), layout.strip_icon(position)) {
                Self::item(item, icon, canvas);
            }
        }
        for slot in 0..Self::SLOTS {
            if let Some(item) = cache.item(slot) {
                Self::durability_bar(item, layout.icon(slot), canvas);
            }
        }
        for slot in 0..Self::SLOTS {
            if let Some(item) = cache.item(slot) {
                Self::fill_bar(item, layout.icon(slot), canvas);
            }
        }
        for position in 0..layout.strip_cells() {
            if let (Some(item), Some(icon)) = (cache.strip_item(position), layout.strip_icon(position)) {
                Self::durability_bar(item, icon, canvas);
                Self::fill_bar(item, icon, canvas);
            }
        }
        let mut queued = false;
        for slot in 0..Self::SLOTS {
            let Some(text) = cache.item(slot).and_then(|item| item.count_text.as_deref()) else { continue };
            canvas.count(text, layout.cell(slot));
            queued = true;
        }
        for position in 0..layout.strip_cells() {
            let Some(text) = cache.strip_item(position).and_then(|item| item.count_text.as_deref()) else { continue };
            let Some(cell) = layout.strip_cell(position) else { continue };
            canvas.count(text, cell);
            queued = true;
        }
        if let (Some(text), Some(area)) = (hint, layout.hint()) {
            canvas.label(text, area);
            queued = true;
        }
        if queued {
            canvas.flush_text();
        }
    }

    fn item<C: Canvas>(item: &PreviewItem<C::Stack>, icon: Rect, canvas: &mut C) {
        canvas.item(&item.stack, icon, false, PanelStyle::ITEM_ALPHA, PanelStyle::ITEM_Z_ORDER);
        if item.glint {
            canvas.item(&item.stack, icon, true, PanelStyle::GLINT_ALPHA, PanelStyle::ITEM_Z_ORDER);
        }
    }

    fn durability_bar<C: Canvas>(item: &PreviewItem<C::Stack>, icon: Rect, canvas: &mut C) {
        let Some(bar) = item.bar else { return };
        canvas.fill(bar.background(icon), PanelStyle::BAR_BACKGROUND, PanelStyle::BAR_ALPHA);
        let foreground = bar.foreground(icon);
        if foreground.width() > 0.0 {
            canvas.fill(foreground, bar.color(), PanelStyle::BAR_ALPHA);
        }
    }

    // a bundle with a weight above 0 shows its fill; an empty bundle and one with no weight show none
    fn fill_bar<C: Canvas>(item: &PreviewItem<C::Stack>, icon: Rect, canvas: &mut C) {
        if !item.bundle {
            return;
        }
        let Some(fraction) = FillBar::fraction(item.weight).filter(|fraction| *fraction > 0.0) else { return };
        canvas.fill(FillBar::background(icon), PanelStyle::BAR_BACKGROUND, PanelStyle::BAR_ALPHA);
        let foreground = FillBar::foreground(icon, fraction);
        if foreground.width() > 0.0 {
            canvas.fill(foreground, PanelStyle::FILL_BAR, PanelStyle::BAR_ALPHA);
        }
    }
}
