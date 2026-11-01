use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use alaydriem_peek::detours::slot_hovered;
use alaydriem_peek::services::PeekRuntime;

use crate::support::RuntimeHarness;

static SEEN: Mutex<Vec<(usize, usize, i32, bool)>> = Mutex::new(Vec::new());

extern "C" fn game_hovered(this: usize, collection: usize, slot: i32) -> u32 {
    let tracked = PeekRuntime::get().is_some_and(|runtime| runtime.is_tracking(this));
    SEEN.lock().unwrap().push((this, collection, slot, tracked));
    0xAB
}

#[test]
fn the_detour_passes_its_arguments_on_unchanged_and_returns_the_original_result() {
    slot_hovered::GUARD.original().store(game_hovered as *mut c_void, Ordering::SeqCst);

    let result = slot_hovered::detour(0x1000, 0x2000, 7);

    assert_eq!(result, 0xAB);
    assert_eq!(SEEN.lock().unwrap().clone(), [(0x1000, 0x2000, 7, false)]);
}

#[test]
fn the_slot_is_tracked_after_the_original_ran() {
    let runtime = RuntimeHarness::start(&[]);
    let collection = RuntimeHarness::collection("container_items");
    slot_hovered::GUARD.original().store(game_hovered as *mut c_void, Ordering::SeqCst);

    let result = slot_hovered::detour(0x1000, collection, 7);

    assert_eq!(result, 0xAB);
    assert_eq!(SEEN.lock().unwrap().clone(), [(0x1000, collection, 7, false)]);
    assert!(runtime.is_tracking(0x1000));
}
