use peek_core::services::HoverTracker;
use peek_core::structs::SlotCollection;

#[test]
fn unhover_of_the_old_slot_after_hover_of_the_new_keeps_the_new() {
    let mut tracker = HoverTracker::new();
    tracker.hovered(1, SlotCollection::Inventory, 3);
    tracker.hovered(1, SlotCollection::Inventory, 4);
    tracker.unhovered(1, SlotCollection::Inventory, 3);
    let current = tracker.current_for(1).unwrap();
    assert_eq!(current.index, 4);
    assert_eq!(current.collection, SlotCollection::Inventory);
}

#[test]
fn unhover_of_the_current_slot_clears_it() {
    let mut tracker = HoverTracker::new();
    tracker.hovered(1, SlotCollection::Inventory, 3);
    tracker.unhovered(1, SlotCollection::Inventory, 3);
    assert!(tracker.current_for(1).is_none());
}

#[test]
fn unhover_from_another_screen_or_collection_is_ignored() {
    let mut tracker = HoverTracker::new();
    tracker.hovered(1, SlotCollection::Inventory, 3);
    tracker.unhovered(2, SlotCollection::Inventory, 3);
    tracker.unhovered(1, SlotCollection::Container, 3);
    assert_eq!(tracker.current_for(1).unwrap().index, 3);
}

#[test]
fn leaving_the_screen_clears_its_slot() {
    let mut tracker = HoverTracker::new();
    tracker.hovered(1, SlotCollection::Inventory, 3);
    tracker.left(1);
    assert!(tracker.current_for(1).is_none());
}

#[test]
fn leaving_another_screen_keeps_the_slot() {
    let mut tracker = HoverTracker::new();
    tracker.hovered(1, SlotCollection::Inventory, 3);
    tracker.left(2);
    assert_eq!(tracker.current_for(1).unwrap().index, 3);
    assert!(tracker.current_for(2).is_none());
}

#[test]
fn clear_drops_everything() {
    let mut tracker = HoverTracker::new();
    tracker.hovered(1, SlotCollection::Inventory, 3);
    tracker.clear();
    assert!(tracker.current_for(1).is_none());
}

#[test]
fn the_screen_is_the_screen_of_the_current_slot() {
    let mut tracker = HoverTracker::new();
    assert_eq!(tracker.screen(), None);

    tracker.hovered(0x10, SlotCollection::Container, 3);
    assert_eq!(tracker.screen(), Some(0x10));

    tracker.left(0x10);
    assert_eq!(tracker.screen(), None);
}
