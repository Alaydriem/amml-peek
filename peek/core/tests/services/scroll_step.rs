use peek_core::services::ScrollStep;

#[test]
fn wheel_down_is_forward_and_wheel_up_is_back() {
    assert_eq!(ScrollStep::notches(-0.94), 1);
    assert_eq!(ScrollStep::notches(0.94), -1);
}

#[test]
fn no_movement_is_no_step() {
    assert_eq!(ScrollStep::notches(0.0), 0);
    assert_eq!(ScrollStep::notches(f32::NAN), 0);
}
