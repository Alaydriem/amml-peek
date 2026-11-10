use alaydriem_peek::services::ScrollGate;

#[test]
fn the_goal_case_is_consumed() {
    assert!(ScrollGate::consume(Some(0x10), Some(0x10), 2, true, 1));
    assert!(ScrollGate::consume(Some(0x10), Some(0x10), 27, true, -1));
}

#[test]
fn no_hover_or_no_scene_controller_is_not_consumed() {
    assert!(!ScrollGate::consume(None, Some(0x10), 2, true, 1));
    assert!(!ScrollGate::consume(Some(0x10), None, 2, true, 1));
    assert!(!ScrollGate::consume(None, None, 2, true, 1));
}

#[test]
fn another_screen_is_not_consumed() {
    assert!(!ScrollGate::consume(Some(0x10), Some(0x20), 2, true, 1));
}

#[test]
fn fewer_than_two_bundles_is_not_consumed() {
    assert!(!ScrollGate::consume(Some(0x10), Some(0x10), 0, true, 1));
    assert!(!ScrollGate::consume(Some(0x10), Some(0x10), 1, true, 1));
}

#[test]
fn without_shift_is_not_consumed() {
    assert!(!ScrollGate::consume(Some(0x10), Some(0x10), 2, false, 1));
}

#[test]
fn a_notch_that_does_not_move_is_not_consumed() {
    assert!(!ScrollGate::consume(Some(0x10), Some(0x10), 2, true, 0));
}
