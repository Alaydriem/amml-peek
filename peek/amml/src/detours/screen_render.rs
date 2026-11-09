use amml_sdk::DetourGuard;

use crate::services::PeekRuntime;

// void ScreenView::render(UIRenderContext&). The original draws the screen; the panel is then drawn on top of it
pub static GUARD: DetourGuard = DetourGuard::new("screen_render");

// the signature of the hooked function: the view and the UI render context
pub type ScreenRenderFn = unsafe extern "C" fn(usize, usize);

#[allow(unsafe_code)]
pub extern "C" fn detour(this: usize, ctx: usize) {
    let original: ScreenRenderFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature; the arguments are the game's
    unsafe { original(this, ctx) };
    if let Some(runtime) = PeekRuntime::get() {
        runtime.on_render(this, ctx);
    }
}
