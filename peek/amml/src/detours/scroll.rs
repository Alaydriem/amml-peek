use amml_sdk::DetourGuard;

use crate::services::PeekRuntime;

// void UIScene::handleDirection(DirectionId, float, float, FocusImpact). One wheel notch is one call. The runtime
// decides whether peek takes it; when it does the original does not run, and the game never sees that notch
pub static GUARD: DetourGuard = DetourGuard::new("scroll");

// the signature of the hooked function: the scene, the DirectionId, the step x and y, and the FocusImpact
pub type ScrollFn = unsafe extern "C" fn(usize, u8, f32, f32, u8);

#[allow(unsafe_code)]
pub extern "C" fn detour(scene: usize, direction: u8, x: f32, y: f32, focus_impact: u8) {
    if PeekRuntime::get().is_some_and(|runtime| runtime.on_scroll(scene, direction, x, y)) {
        return;
    }
    let original: ScrollFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature; the arguments are the game's
    unsafe { original(scene, direction, x, y, focus_impact) };
}
