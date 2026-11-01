#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoveredSlot {
    pub controller: usize,
    pub collection: String,
    pub index: i32,
}
