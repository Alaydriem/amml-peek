use peek_core::structs::ShulkerGrid;

#[test]
fn empty_grid_has_nothing_filled() {
    let grid = ShulkerGrid::new();
    assert_eq!(grid.filled(), 0);
    assert_eq!(grid.entry_at(0), None);
    assert_eq!(ShulkerGrid::COLUMNS * ShulkerGrid::ROWS, 27);
}

#[test]
fn out_of_range_slots_are_refused() {
    let mut grid = ShulkerGrid::new();
    assert!(!grid.place(-1, 0));
    assert!(!grid.place(27, 0));
    assert!(!grid.place(i32::MIN, 0));
    assert!(!grid.place(i32::MAX, 0));
    assert_eq!(grid.filled(), 0);
}

#[test]
fn last_slot_is_accepted() {
    let mut grid = ShulkerGrid::new();
    assert!(grid.place(26, 5));
    assert_eq!(grid.entry_at(26), Some(5));
    assert_eq!(grid.filled(), 1);
}

#[test]
fn later_entry_replaces_earlier_for_the_same_slot() {
    let mut grid = ShulkerGrid::new();
    assert!(grid.place(3, 1));
    assert!(grid.place(3, 2));
    assert_eq!(grid.entry_at(3), Some(2));
    assert_eq!(grid.filled(), 1);
}

#[test]
fn entry_at_out_of_range_is_none() {
    let grid = ShulkerGrid::new();
    assert_eq!(grid.entry_at(27), None);
}
