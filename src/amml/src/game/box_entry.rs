use std::sync::Arc;

use amml_api::ItemDrawHandle;
use peek_core::structs::DurabilityBar;

/// One entry of a shulker box or a bundle, copied from the contents of the loader: the fields the draw needs and the
/// handle the loader draws the stack from. The handle is shared, so the same entry feeds the grid and the strip.
pub struct BoxEntry {
    pub slot: u8,
    pub count: u8,
    pub glint: bool,
    pub bar: Option<DurabilityBar>,
    pub bundle: bool,
    pub weight: Option<i32>,
    pub draw: Arc<ItemDrawHandle>,
    pub contents: Option<Arc<[BoxEntry]>>,
}
