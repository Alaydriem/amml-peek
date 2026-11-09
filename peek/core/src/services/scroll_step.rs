pub struct ScrollStep;

impl ScrollStep {
    /// The selection step of one wheel notch from its `y` value: wheel down (negative) is one step forward, wheel
    /// up is one step back, and no movement is no step.
    pub fn notches(y: f32) -> i32 {
        if y < 0.0 {
            1
        } else if y > 0.0 {
            -1
        } else {
            0
        }
    }
}
