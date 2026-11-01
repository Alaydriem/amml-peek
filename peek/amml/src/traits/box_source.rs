use crate::types::PreviewItem;

/// The entries of one shulker box or one bundle, as they are stored: a list in which each entry carries its own slot.
pub trait BoxSource {
    /// What a built item holds for as long as it is shown.
    type Stack;

    /// The entries of a bundle in this list.
    type Contents: BoxSource<Stack = Self::Stack>;

    /// The number of entries in the list.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The `Slot` of the entry at `index`; `None` when it has none.
    fn slot(&self, index: usize) -> Option<i32>;

    /// The item of the entry at `index`; `None` when the game cannot make an item of it.
    fn build(&self, index: usize) -> Option<PreviewItem<Self::Stack>>;

    /// The entries inside the bundle at `index`. `None` when the entry is not a bundle, or its list is missing or
    /// not a list; the entries of the list that are not usable are left to `slot` and `build`.
    fn bundle_contents(&self, index: usize) -> Option<Self::Contents>;
}
