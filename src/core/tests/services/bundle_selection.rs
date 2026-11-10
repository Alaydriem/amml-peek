use peek_core::services::BundleSelection;

fn selection(slots: &[u8]) -> BundleSelection {
    let mut selection = BundleSelection::new();
    selection.reset(slots);
    selection
}

#[test]
fn the_first_slot_is_selected_at_the_start() {
    let selection = selection(&[4, 9, 20]);

    assert_eq!(selection.selected(), Some(4));
    assert_eq!(selection.position(), Some((1, 3)));
}

#[test]
fn no_bundles_select_nothing() {
    let selection = selection(&[]);

    assert_eq!(selection.selected(), None);
    assert_eq!(selection.position(), None);
}

#[test]
fn step_moves_and_wraps_in_both_directions() {
    let mut selection = selection(&[4, 9, 20]);

    assert!(selection.step(1));
    assert_eq!(selection.selected(), Some(9));
    assert!(selection.step(1));
    assert!(selection.step(1));
    assert_eq!(selection.selected(), Some(4));
    assert!(selection.step(-1));
    assert_eq!(selection.selected(), Some(20));
    assert!(selection.step(-1));
    assert_eq!(selection.selected(), Some(9));
}

#[test]
fn a_step_of_a_whole_turn_changes_nothing() {
    let mut selection = selection(&[4, 9, 20]);

    assert!(!selection.step(3));
    assert_eq!(selection.selected(), Some(4));
    assert!(!selection.step(0));
}

#[test]
fn with_fewer_than_two_bundles_step_changes_nothing() {
    let mut one = selection(&[7]);
    let mut none = selection(&[]);

    assert!(!one.step(1));
    assert!(!one.step(-1));
    assert_eq!(one.selected(), Some(7));
    assert!(!none.step(1));
}

#[test]
fn reset_starts_again_at_the_first_of_the_new_slots() {
    let mut selection = selection(&[4, 9, 20]);
    selection.step(2);

    selection.reset(&[1, 2]);

    assert_eq!(selection.selected(), Some(1));
    assert_eq!(selection.position(), Some((1, 2)));
}

#[test]
fn position_counts_from_one() {
    let mut selection = selection(&[1, 2, 3, 4, 5]);
    selection.step(1);

    assert_eq!(selection.position(), Some((2, 5)));
}

#[test]
fn more_than_27_slots_keep_the_first_27() {
    let slots: Vec<u8> = (0..40).collect();
    let mut selection = selection(&slots);

    assert_eq!(selection.count(), 27);
    selection.step(-1);
    assert_eq!(selection.selected(), Some(26));
}
