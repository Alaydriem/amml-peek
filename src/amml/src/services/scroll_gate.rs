/// Whether a wheel notch is taken from the game.
pub struct ScrollGate;

impl ScrollGate {
    /// True only for a notch over a shulker box with two or more bundles, with Shift held, that reaches the scene of
    /// the screen the pointer is on: both controllers are known and the same, and the notch moves.
    pub fn consume(
        hovered_controller: Option<usize>,
        scene_controller: Option<usize>,
        bundles: usize,
        shift: bool,
        notches: i32,
    ) -> bool {
        let same_screen =
            matches!((hovered_controller, scene_controller), (Some(hovered), Some(scene)) if hovered == scene);
        same_screen && bundles >= 2 && shift && notches != 0
    }
}
