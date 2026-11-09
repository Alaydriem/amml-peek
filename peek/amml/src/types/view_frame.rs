use peek_core::structs::Point;

/// The screen a render call draws: its controller, the pointer and the size, in GUI units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewFrame {
    pub controller: usize,
    pub pointer: Point,
    pub screen: Point,
}
