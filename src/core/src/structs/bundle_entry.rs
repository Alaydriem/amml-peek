/// A bundle inside a shulker box: the box slot it sits in and its fill weight, if the entry carries one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundleEntry {
    pub slot: u8,
    pub weight: Option<i32>,
}
