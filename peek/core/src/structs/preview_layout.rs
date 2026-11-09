use crate::services::StripLayout;
use crate::structs::{Point, Rect};

/// Positions a grid of slots, an optional strip under it and an optional hint line inside a framed box, in GUI
/// units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreviewLayout {
    columns: u8,
    frame: Rect,
    rows: u8,
    strip_rows: usize,
    hint: bool,
}

impl PreviewLayout {
    pub const CELL: f32 = 18.0;
    pub const ICON: f32 = 16.0;
    pub const PADDING: f32 = 4.0;
    pub const GAP: f32 = 8.0;
    /// The height of the hint line.
    pub const HINT_HEIGHT: f32 = 8.0;
    /// The space between the last cell row and the hint line.
    pub const HINT_GAP: f32 = 4.0;

    /// Places the frame of a grid alone: see `anchored_panel`.
    pub fn anchored(columns: u8, rows: u8, pointer: Point, screen: Point) -> PreviewLayout {
        Self::anchored_panel(columns, rows, 0, false, pointer, screen)
    }

    /// Places the whole panel, the grid, then `strip_rows` rows under it (none for 0) and the hint line when asked.
    /// It goes above and right of the pointer, left of it when it runs off the right edge and below when it runs
    /// off the top. Clamping into the screen is the last resort.
    pub fn anchored_panel(
        columns: u8,
        rows: u8,
        strip_rows: usize,
        hint: bool,
        pointer: Point,
        screen: Point,
    ) -> PreviewLayout {
        let width = f32::from(columns) * Self::CELL + 2.0 * Self::PADDING;
        let height = Self::height(rows, strip_rows, hint);

        let mut x = pointer.x + Self::GAP;
        if x + width > screen.x {
            x = pointer.x - Self::GAP - width;
        }
        if x < 0.0 {
            x = (pointer.x + Self::GAP).min(screen.x - width).max(0.0);
        }

        let mut y = pointer.y - Self::GAP - height;
        if y < 0.0 {
            y = pointer.y + Self::GAP;
        }
        if y + height > screen.y {
            y = (screen.y - height).max(0.0);
        }

        PreviewLayout { columns, frame: Rect { x0: x, y0: y, x1: x + width, y1: y + height }, rows, strip_rows, hint }
    }

    fn height(rows: u8, strip_rows: usize, hint: bool) -> f32 {
        let mut height = f32::from(rows) * Self::CELL + 2.0 * Self::PADDING;
        if strip_rows > 0 {
            height += Self::GAP + strip_rows as f32 * Self::CELL;
        }
        if hint {
            height += Self::HINT_GAP + Self::HINT_HEIGHT;
        }
        height
    }

    pub fn frame(&self) -> Rect {
        self.frame
    }

    /// Full cell of `slot`, row-major.
    pub fn cell(&self, slot: usize) -> Rect {
        let columns = usize::from(self.columns).max(1);
        self.cell_at(self.frame.y0 + Self::PADDING, slot % columns, slot / columns)
    }

    /// Icon area centred in the cell of `slot`.
    pub fn icon(&self, slot: usize) -> Rect {
        Self::centred(self.cell(slot))
    }

    /// The number of cells the strip draws: whole rows, at most `StripLayout::MAX_ENTRIES`.
    pub fn strip_cells(&self) -> usize {
        (self.strip_rows * StripLayout::COLUMNS).min(StripLayout::MAX_ENTRIES)
    }

    /// Full cell of strip entry `index`; `None` without a strip or from `StripLayout::MAX_ENTRIES` on.
    pub fn strip_cell(&self, index: usize) -> Option<Rect> {
        if self.strip_rows == 0 {
            return None;
        }
        let (column, row) = StripLayout::cell(index)?;
        let top = self.frame.y0 + Self::PADDING + f32::from(self.rows) * Self::CELL + Self::GAP;
        Some(self.cell_at(top, column, row))
    }

    /// Icon area centred in strip entry `index`.
    pub fn strip_icon(&self, index: usize) -> Option<Rect> {
        self.strip_cell(index).map(Self::centred)
    }

    /// The line under the cells; `None` when the layout has no hint.
    pub fn hint(&self) -> Option<Rect> {
        self.hint.then_some(Rect {
            x0: self.frame.x0 + Self::PADDING,
            y0: self.frame.y1 - Self::PADDING - Self::HINT_HEIGHT,
            x1: self.frame.x1 - Self::PADDING,
            y1: self.frame.y1 - Self::PADDING,
        })
    }

    fn cell_at(&self, top: f32, column: usize, row: usize) -> Rect {
        let x = self.frame.x0 + Self::PADDING + column as f32 * Self::CELL;
        let y = top + row as f32 * Self::CELL;
        Rect { x0: x, y0: y, x1: x + Self::CELL, y1: y + Self::CELL }
    }

    fn centred(cell: Rect) -> Rect {
        let inset = (Self::CELL - Self::ICON) / 2.0;
        Rect {
            x0: cell.x0 + inset,
            y0: cell.y0 + inset,
            x1: cell.x0 + inset + Self::ICON,
            y1: cell.y0 + inset + Self::ICON,
        }
    }
}
