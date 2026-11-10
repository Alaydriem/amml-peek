/// What one render call did for the preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawOutcome {
    /// The pointer is not over a slot of this screen.
    Idle,
    /// The slot holds no shulker box.
    Skipped,
    /// The panel was drawn.
    Drawn,
}
