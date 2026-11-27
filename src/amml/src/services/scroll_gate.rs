/// Whether a wheel notch is taken from the game.
pub struct ScrollGate;

impl ScrollGate {
    /// True only for a notch over a shulker box with two or more bundles, with Shift held, that reaches the scene of
    /// the screen the pointer is on: both screens are known and the same, and the notch moves.
    pub fn consume(
        hovered_screen: Option<u64>,
        scene_screen: Option<u64>,
        bundles: usize,
        shift: bool,
        notches: i32,
    ) -> bool {
        let same_screen = matches!((hovered_screen, scene_screen), (Some(hovered), Some(scene)) if hovered == scene);
        same_screen && bundles >= 2 && shift && notches != 0
    }
}
