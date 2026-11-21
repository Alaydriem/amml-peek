use peek_core::structs::HoveredSlot;

use crate::traits::BoxSource;
use crate::types::{PreviewKey, ResolvedBox};

/// Finds what the hovered slot holds.
pub trait BoxResolver {
    type Source: BoxSource;

    /// The key of the stack in `slot`; `None` when the slot is empty. It is read every frame, so it makes no
    /// allocation and reads no contents.
    fn key(&self, slot: &HoveredSlot) -> Option<PreviewKey>;

    /// The shulker box in `slot`, read when the key changes; `None` when the slot holds anything else.
    fn load(&self, slot: &HoveredSlot) -> Option<ResolvedBox<Self::Source>>;
}
