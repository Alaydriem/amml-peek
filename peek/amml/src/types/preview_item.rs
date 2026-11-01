use peek_core::structs::DurabilityBar;

/// One item of the preview with what the draw needs besides the stack, read once when the item is built.
pub struct PreviewItem<S> {
    pub stack: S,
    pub glint: bool,
    pub bar: Option<DurabilityBar>,
    /// The stack count as text, for a stack of more than one. The text stays alive as long as the item does, which
    /// is until after the game drew the queued counts.
    pub count_text: Option<String>,
    /// Whether the item is a bundle.
    pub bundle: bool,
    /// The `bundle_weight` of a bundle that carries one.
    pub weight: Option<i32>,
}
