use peek_core::structs::HoveredSlot;

use crate::traits::BoxSource;
use crate::types::ResolvedBox;

/// Finds what the hovered slot holds.
pub trait BoxResolver {
    type Source: BoxSource;

    /// The shulker box in `slot`; `None` when the slot is empty or holds anything else.
    fn resolve(&self, slot: &HoveredSlot) -> Option<ResolvedBox<Self::Source>>;
}
