/// What identifies the contents of a hovered box: the stack, its user data and its count. The same key means the
/// same contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewKey {
    pub stack: usize,
    pub user_data: usize,
    pub count: u8,
}
