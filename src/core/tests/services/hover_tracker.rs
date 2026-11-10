use peek_core::services::HoverTracker;

#[test]
fn unhover_of_the_old_slot_after_hover_of_the_new_keeps_the_new() {
    let mut tracker = HoverTracker::new();
    tracker.hovered(1, "inventory_items", 3);
    tracker.hovered(1, "inventory_items", 4);
    tracker.unhovered(1, "inventory_items", 3);
    let current = tracker.current_for(1).unwrap();
    assert_eq!(current.index, 4);
    assert_eq!(current.collection, "inventory_items");
}

#[test]
fn unhover_of_the_current_slot_clears_it() {
    let mut tracker = HoverTracker::new();
    tracker.hovered(1, "inventory_items", 3);
    tracker.unhovered(1, "inventory_items", 3);
    assert!(tracker.current_for(1).is_none());
}

#[test]
fn unhover_from_another_controller_or_collection_is_ignored() {
    let mut tracker = HoverTracker::new();
    tracker.hovered(1, "inventory_items", 3);
    tracker.unhovered(2, "inventory_items", 3);
    tracker.unhovered(1, "container_items", 3);
    assert_eq!(tracker.current_for(1).unwrap().index, 3);
}

#[test]
fn leaving_the_screen_clears_its_slot() {
    let mut tracker = HoverTracker::new();
    tracker.hovered(1, "inventory_items", 3);
    tracker.left(1);
    assert!(tracker.current_for(1).is_none());
}

#[test]
fn leaving_another_screen_keeps_the_slot() {
    let mut tracker = HoverTracker::new();
    tracker.hovered(1, "inventory_items", 3);
    tracker.left(2);
    assert_eq!(tracker.current_for(1).unwrap().index, 3);
    assert!(tracker.current_for(2).is_none());
}

#[test]
fn clear_drops_everything() {
    let mut tracker = HoverTracker::new();
    tracker.hovered(1, "inventory_items", 3);
    tracker.clear();
    assert!(tracker.current_for(1).is_none());
}

#[test]
fn the_controller_is_the_screen_of_the_current_slot() {
    let mut tracker = HoverTracker::new();
    assert_eq!(tracker.controller(), None);

    tracker.hovered(0x10, "container_items", 3);
    assert_eq!(tracker.controller(), Some(0x10));

    tracker.left(0x10);
    assert_eq!(tracker.controller(), None);
}
