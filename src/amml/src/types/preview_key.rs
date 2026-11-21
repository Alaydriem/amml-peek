use amml_api::{ScreenId, SlotAt};

/// What identifies the contents of a hovered box: the screen, the slot, the stack identity (none for a stack with user
/// data, which every filled box has), the aux value, the count and the change stamp of the user data (none on a loader without it). The same key means the same contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewKey {
    pub screen: ScreenId,
    pub slot: SlotAt,
    pub identity: Option<u64>,
    pub aux: Option<i16>,
    pub count: u8,
    pub change_stamp: Option<u64>,
}
