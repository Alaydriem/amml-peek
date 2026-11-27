use peek_core::services::ScrollStep;

#[test]
fn wheel_down_is_forward_and_wheel_up_is_back() {
    assert_eq!(ScrollStep::from_notches(-1), 1);
    assert_eq!(ScrollStep::from_notches(3), -1);
}

#[test]
fn no_movement_is_no_step() {
    assert_eq!(ScrollStep::from_notches(0), 0);
}
