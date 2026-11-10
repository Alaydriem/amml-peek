use peek_core::structs::Rect;

pub struct Approx;

impl Approx {
    pub fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    pub fn rect(rect: Rect, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
        Self::near(rect.x0, x0) && Self::near(rect.y0, y0) && Self::near(rect.x1, x1) && Self::near(rect.y1, y1)
    }
}
