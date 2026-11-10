use amml_sdk::DetourGuard;

use crate::services::PeekRuntime;

// void ScreenController::~ScreenController(). The runtime runs first, because the controller is gone after the
// original; it reads nothing through `this`, only compares the address
pub static GUARD: DetourGuard = DetourGuard::new("screen_controller_dtor");

// the signature of the hooked function: this only
pub type ScreenControllerDtorFn = unsafe extern "C" fn(usize);

#[allow(unsafe_code)]
pub extern "C" fn detour(this: usize) {
    if let Some(runtime) = PeekRuntime::get() {
        runtime.on_controller_dtor(this);
    }
    let original: ScreenControllerDtorFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature; `this` is the game's
    unsafe { original(this) };
}
