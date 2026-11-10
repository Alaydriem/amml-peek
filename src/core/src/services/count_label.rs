pub struct CountLabel;

impl CountLabel {
    /// The stack count text, none for an empty or single item.
    pub fn text(count: u8) -> Option<String> {
        (count > 1).then(|| count.to_string())
    }
}
