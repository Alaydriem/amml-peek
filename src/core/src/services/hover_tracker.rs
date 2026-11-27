use crate::structs::{HoveredSlot, SlotCollection};

/// Tracks the one slot the pointer is over. An unhover only clears the slot it names, so a
/// hover of the next slot that arrives before the unhover of the last one stays current.
#[derive(Debug, Default)]
pub struct HoverTracker {
    current: Option<HoveredSlot>,
}

impl HoverTracker {
    pub fn new() -> HoverTracker {
        HoverTracker { current: None }
    }

    pub fn hovered(&mut self, screen: u64, collection: SlotCollection, index: i32) {
        self.current = Some(HoveredSlot { screen, collection, index });
    }

    pub fn unhovered(&mut self, screen: u64, collection: SlotCollection, index: i32) {
        if self.current == Some(HoveredSlot { screen, collection, index }) {
            self.current = None;
        }
    }

    /// The screen `screen` closed.
    pub fn left(&mut self, screen: u64) {
        if self.current.is_some_and(|slot| slot.screen == screen) {
            self.current = None;
        }
    }

    pub fn clear(&mut self) {
        self.current = None;
    }

    /// The screen the current slot is on.
    pub fn screen(&self) -> Option<u64> {
        self.current.map(|slot| slot.screen)
    }

    pub fn current_for(&self, screen: u64) -> Option<&HoveredSlot> {
        self.current.as_ref().filter(|slot| slot.screen == screen)
    }
}
