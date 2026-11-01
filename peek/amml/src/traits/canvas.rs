use peek_core::structs::{Rect, Rgb};

/// What the preview draws with. The game draws the fills, the outline and the items at once; the counts and labels
/// are queued until `flush_text`.
pub trait Canvas {
    /// The item type the canvas draws.
    type Stack;

    fn fill(&mut self, area: Rect, color: Rgb, alpha: f32);

    fn outline(&mut self, area: Rect, color: Rgb, alpha: f32, thickness: i32);

    /// The picture of `stack` in `icon`; `foil` draws only the glint overlay.
    fn item(&mut self, stack: &Self::Stack, icon: Rect, foil: bool, alpha: f32, z_order: i32);

    /// Queues `text` right aligned at the bottom of `cell`. `text` must stay alive and unmoved until `flush_text`.
    fn count(&mut self, text: &str, cell: Rect);

    /// Queues `text` left aligned at the top of `area`. `text` must stay alive and unmoved until `flush_text`.
    fn label(&mut self, text: &str, area: Rect);

    /// Draws the queued text.
    fn flush_text(&mut self);
}
