use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use alaydriem_peek::detours::{slot_hovered, slot_unhovered};
use alaydriem_peek::services::PeekRuntime;

use crate::support::RuntimeHarness;

static SEEN: Mutex<Vec<(usize, usize, i32, bool)>> = Mutex::new(Vec::new());

extern "C" fn game_unhovered(this: usize, collection: usize, slot: i32) -> u32 {
    let tracked = PeekRuntime::get().is_some_and(|runtime| runtime.is_tracking(this));
    SEEN.lock().unwrap().push((this, collection, slot, tracked));
    0xCD
}

extern "C" fn game_hovered(_: usize, _: usize, _: i32) -> u32 {
    0
}

#[test]
fn the_detour_passes_its_arguments_on_unchanged_and_returns_the_original_result() {
    slot_unhovered::GUARD.original().store(game_unhovered as *mut c_void, Ordering::SeqCst);

    let result = slot_unhovered::detour(0x1000, 0x2000, 7);

    assert_eq!(result, 0xCD);
    assert_eq!(SEEN.lock().unwrap().clone(), [(0x1000, 0x2000, 7, false)]);
}

#[test]
fn the_slot_is_forgotten_after_the_original_ran() {
    let runtime = RuntimeHarness::start(&[]);
    let collection = RuntimeHarness::collection("container_items");
    slot_hovered::GUARD.original().store(game_hovered as *mut c_void, Ordering::SeqCst);
    slot_unhovered::GUARD.original().store(game_unhovered as *mut c_void, Ordering::SeqCst);
    slot_hovered::detour(0x1000, collection, 7);
    assert!(runtime.is_tracking(0x1000));

    let result = slot_unhovered::detour(0x1000, collection, 7);

    assert_eq!(result, 0xCD);
    assert_eq!(SEEN.lock().unwrap().clone(), [(0x1000, collection, 7, true)]);
    assert!(!runtime.is_tracking(0x1000));
}
