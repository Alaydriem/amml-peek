pub struct ScrollStep;

impl ScrollStep {
    /// The selection step of one wheel step from the UI scroll count, which is positive for wheel up: wheel down is
    /// one step forward, wheel up is one step back, and no movement is no step.
    pub fn from_notches(notches: i32) -> i32 {
        -notches.signum()
    }
}
