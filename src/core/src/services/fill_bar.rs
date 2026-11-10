use crate::structs::{DurabilityBar, Rect};

/// The fill bar of a bundle cell, drawn where the durability bar goes.
pub struct FillBar;

impl FillBar {
    /// The weight a full bundle holds.
    pub const CAPACITY: i32 = 64;

    /// Filled share in 0 to 1; `None` without a weight or with a negative one.
    pub fn fraction(weight: Option<i32>) -> Option<f32> {
        let weight = weight.filter(|weight| *weight >= 0)?;
        Some((weight as f32 / Self::CAPACITY as f32).clamp(0.0, 1.0))
    }

    /// Black strip under the fill, the same place and size as the durability bar.
    pub fn background(icon: Rect) -> Rect {
        DurabilityBar::strip(icon)
    }

    /// Fill on the top row of the strip, left aligned, a whole number of units wide.
    pub fn foreground(icon: Rect, fraction: f32) -> Rect {
        let background = Self::background(icon);
        let units = if fraction > 0.0 { (DurabilityBar::FOREGROUND_WIDTH * fraction + 0.5).floor() } else { 0.0 };
        let units = units.min(DurabilityBar::FOREGROUND_WIDTH);
        Rect {
            x0: background.x0,
            y0: background.y0,
            x1: background.x0 + units,
            y1: background.y0 + DurabilityBar::FOREGROUND_HEIGHT,
        }
    }
}
