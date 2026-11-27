use peek_core::structs::{Rect, Rgb};

/// What the preview draws with. The fills, the outline and the items draw at once; the counts and labels are queued
/// text that the loader draws over everything after the last listener.
pub trait Canvas {
    /// The item type the canvas draws.
    type Stack;

    fn fill(&mut self, area: Rect, color: Rgb, alpha: f32);

    fn outline(&mut self, area: Rect, color: Rgb, alpha: f32, thickness: i32);

    /// The picture of `stack` in `icon`; `foil` draws only the glint overlay.
    fn item(&mut self, stack: &Self::Stack, icon: Rect, foil: bool, alpha: f32, z_order: i32);

    /// Draws `text` right aligned at the bottom of `cell`.
    fn count(&mut self, text: &str, cell: Rect);

    /// Draws `text` left aligned at the top of `area`.
    fn label(&mut self, text: &str, area: Rect);
}
