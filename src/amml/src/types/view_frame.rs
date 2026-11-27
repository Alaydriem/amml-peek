use peek_core::structs::Point;

/// The screen a render call draws: its id, the pointer and the size, in GUI units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewFrame {
    pub screen: u64,
    pub pointer: Point,
    pub size: Point,
}
