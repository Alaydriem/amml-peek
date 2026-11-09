use amml_sdk::DetourGuard;

use crate::services::PeekRuntime;

// void ShulkerBoxBlockItem::appendFormattedHovertext(ItemStackBase const&, Level&, RedactableString&, bool). The
// override calls the base function and then appends the contents lines. While peek is on the base function is called
// instead, with the same arguments, so those lines are never added; otherwise the original runs
pub static GUARD: DetourGuard = DetourGuard::new("shulker_hovertext");

// the signature of the hooked function and of the base: this, the stack, the level, the text, and the bool, which
// travels on the stack and is passed on whole
pub type AppendHovertextFn = unsafe extern "C" fn(usize, usize, usize, usize, usize);

#[allow(unsafe_code)]
pub extern "C" fn detour(this: usize, stack: usize, level: usize, text: usize, flag: usize) {
    let original: AppendHovertextFn = GUARD.original_fn();
    let target = PeekRuntime::get().map_or(original as usize, |runtime| runtime.hovertext_target(original as usize));
    // SAFETY: `target` is the original or the base function, which has the signature of the hooked one
    let function: AppendHovertextFn = unsafe { std::mem::transmute::<usize, AppendHovertextFn>(target) };
    // SAFETY: the arguments are the game's, passed on as they came
    unsafe { function(this, stack, level, text, flag) };
}
