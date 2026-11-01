use amml_sdk::DetourGuard;

use crate::services::PeekRuntime;

// ui::ViewRequest ContainerScreenController::_onContainerSlotHovered(std::string const&, int). The original runs
// first; the hover is then noted. The ViewRequest comes back in EAX and is returned unchanged
pub static GUARD: DetourGuard = DetourGuard::new("slot_hovered");

// the signature of the hooked function: this, the collection name, the slot index; the result in EAX
pub type SlotHoveredFn = unsafe extern "C" fn(usize, usize, i32) -> u32;

#[allow(unsafe_code)]
pub extern "C" fn detour(this: usize, collection: usize, slot: i32) -> u32 {
    let original: SlotHoveredFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature; the arguments are the game's
    let result = unsafe { original(this, collection, slot) };
    if let Some(runtime) = PeekRuntime::get() {
        runtime.on_hover(this, collection, slot);
    }
    result
}
