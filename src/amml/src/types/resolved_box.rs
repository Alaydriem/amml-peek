use crate::types::PreviewKey;

/// A shulker box found in the hovered slot. `contents` is `None` for a box with no stored items.
pub struct ResolvedBox<B> {
    pub key: PreviewKey,
    pub contents: Option<B>,
}
