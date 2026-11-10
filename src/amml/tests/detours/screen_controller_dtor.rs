use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use alaydriem_peek::detours::{screen_controller_dtor, slot_hovered};
use alaydriem_peek::services::PeekRuntime;

use crate::support::RuntimeHarness;

static SEEN: Mutex<Vec<(usize, bool)>> = Mutex::new(Vec::new());

extern "C" fn game_dtor(this: usize) {
    let tracked = PeekRuntime::get().is_some_and(|runtime| runtime.is_tracking(this));
    SEEN.lock().unwrap().push((this, tracked));
}

extern "C" fn game_hovered(_: usize, _: usize, _: i32) -> u32 {
    0
}

#[test]
fn the_detour_passes_the_controller_on_unchanged() {
    screen_controller_dtor::GUARD.original().store(game_dtor as *mut c_void, Ordering::SeqCst);

    screen_controller_dtor::detour(0x1000);

    assert_eq!(SEEN.lock().unwrap().clone(), [(0x1000, false)]);
}

#[test]
fn the_hover_of_the_closing_screen_is_forgotten_before_the_original_destroys_it() {
    let runtime = RuntimeHarness::start(&[]);
    let collection = RuntimeHarness::collection("container_items");
    slot_hovered::GUARD.original().store(game_hovered as *mut c_void, Ordering::SeqCst);
    screen_controller_dtor::GUARD.original().store(game_dtor as *mut c_void, Ordering::SeqCst);
    slot_hovered::detour(0x1000, collection, 7);
    assert!(runtime.is_tracking(0x1000));

    screen_controller_dtor::detour(0x1000);

    assert_eq!(SEEN.lock().unwrap().clone(), [(0x1000, false)]);
    assert!(!runtime.is_tracking(0x1000));
}
