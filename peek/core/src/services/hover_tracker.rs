use crate::structs::HoveredSlot;

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

    pub fn hovered(&mut self, controller: usize, collection: &str, index: i32) {
        match &mut self.current {
            Some(slot) => {
                slot.controller = controller;
                slot.collection.clear();
                slot.collection.push_str(collection);
                slot.index = index;
            }
            None => self.current = Some(HoveredSlot { controller, collection: collection.to_owned(), index }),
        }
    }

    pub fn unhovered(&mut self, controller: usize, collection: &str, index: i32) {
        let matches = self
            .current
            .as_ref()
            .is_some_and(|slot| slot.controller == controller && slot.collection == collection && slot.index == index);
        if matches {
            self.current = None;
        }
    }

    /// The screen of `controller` closed.
    pub fn left(&mut self, controller: usize) {
        if self.current.as_ref().is_some_and(|slot| slot.controller == controller) {
            self.current = None;
        }
    }

    pub fn clear(&mut self) {
        self.current = None;
    }

    /// The screen the current slot is on.
    pub fn controller(&self) -> Option<usize> {
        self.current.as_ref().map(|slot| slot.controller)
    }

    pub fn current_for(&self, controller: usize) -> Option<&HoveredSlot> {
        self.current.as_ref().filter(|slot| slot.controller == controller)
    }
}
