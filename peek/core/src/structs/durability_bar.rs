use crate::structs::{Rect, Rgb};

/// Durability overlay of one damaged item, matching the vanilla slot bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DurabilityBar {
    damage: i32,
    max_damage: i32,
}

impl DurabilityBar {
    pub(crate) const LEFT: f32 = 2.0;
    pub(crate) const TOP: f32 = 12.5;
    pub(crate) const BACKGROUND_WIDTH: f32 = 13.0;
    pub(crate) const BACKGROUND_HEIGHT: f32 = 2.0;
    pub(crate) const FOREGROUND_WIDTH: f32 = 12.0;
    pub(crate) const FOREGROUND_HEIGHT: f32 = 1.0;

    /// `None` unless the item is damageable and has taken damage.
    pub fn new(damageable: bool, damage: i32, max_damage: i32) -> Option<DurabilityBar> {
        (damageable && max_damage > 0 && damage > 0).then_some(DurabilityBar { damage, max_damage })
    }

    /// Remaining durability in 0 to 1.
    pub fn ratio(&self) -> f32 {
        let ratio = (self.max_damage - self.damage) as f32 / self.max_damage as f32;
        ratio.clamp(0.0, 1.0)
    }

    /// Black strip under the fill: the bar and its drop shadow.
    pub fn background(&self, icon: Rect) -> Rect {
        Self::strip(icon)
    }

    /// The strip of a bar drawn over `icon`, shared by every bar that sits where the durability bar goes.
    pub(crate) fn strip(icon: Rect) -> Rect {
        let x0 = icon.x0 + Self::LEFT;
        let y0 = icon.y0 + Self::TOP;
        Rect { x0, y0, x1: x0 + Self::BACKGROUND_WIDTH, y1: y0 + Self::BACKGROUND_HEIGHT }
    }

    /// Fill on the top row of the strip, left aligned, a whole number of units wide.
    pub fn foreground(&self, icon: Rect) -> Rect {
        let background = self.background(icon);
        let ratio = self.ratio();
        let units = if ratio > 0.0 { (Self::FOREGROUND_WIDTH * ratio + 0.5).floor() } else { 0.0 };
        let units = units.min(Self::FOREGROUND_WIDTH);
        Rect {
            x0: background.x0,
            y0: background.y0,
            x1: background.x0 + units,
            y1: background.y0 + Self::FOREGROUND_HEIGHT,
        }
    }

    /// Hue is the ratio divided by three, so red when empty and green when full.
    pub fn color(&self) -> Rgb {
        let hue = self.ratio() / 3.0;
        if hue <= 0.0 {
            return Rgb { r: 1.0, g: 0.0, b: 0.0 };
        }
        if hue >= 1.0 / 3.0 {
            return Rgb { r: 0.0, g: 1.0, b: 0.0 };
        }
        let sector = hue * 6.0;
        let whole = sector.floor();
        let f = sector - whole;
        if whole < 1.0 { Rgb { r: 1.0, g: f, b: 0.0 } } else { Rgb { r: 1.0 - f, g: 1.0, b: 0.0 } }
    }
}
