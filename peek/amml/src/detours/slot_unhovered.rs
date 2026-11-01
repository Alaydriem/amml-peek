use amml_sdk::DetourGuard;

use crate::services::PeekRuntime;

// ui::ViewRequest ContainerScreenController::_onContainerSlotUnhovered(std::string const&, int). The original runs
// first; the slot is then forgotten. The ViewRequest comes back in EAX and is returned unchanged
pub static GUARD: DetourGuard = DetourGuard::new("slot_unhovered");

// the signature of the hooked function: this, the collection name, the slot index; the result in EAX
pub type SlotUnhoveredFn = unsafe extern "C" fn(usize, usize, i32) -> u32;

#[allow(unsafe_code)]
pub extern "C" fn detour(this: usize, collection: usize, slot: i32) -> u32 {
    let original: SlotUnhoveredFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature; the arguments are the game's
    let result = unsafe { original(this, collection, slot) };
    if let Some(runtime) = PeekRuntime::get() {
        runtime.on_unhover(this, collection, slot);
    }
    result
}
