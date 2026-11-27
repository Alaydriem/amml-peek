use crate::structs::SlotCollection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoveredSlot {
    pub screen: u64,
    pub collection: SlotCollection,
    pub index: i32,
}
