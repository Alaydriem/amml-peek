use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use alaydriem_peek::detours::{screen_render, slot_hovered};
use alaydriem_peek::services::PeekRuntime;
use amm_client::test_support::FakeTable;
use amm_core::structs::event::ModEvent;

use crate::support::RuntimeHarness;

// this, the context, and whether the mod was on when the original ran
static SEEN: Mutex<Vec<(usize, usize, bool)>> = Mutex::new(Vec::new());

extern "C" fn game_render(this: usize, ctx: usize) {
    let on = PeekRuntime::get().is_some_and(PeekRuntime::is_on);
    SEEN.lock().unwrap().push((this, ctx, on));
}

extern "C" fn game_hovered(_: usize, _: usize, _: i32) -> u32 {
    0
}

#[test]
fn the_detour_passes_the_view_and_the_context_on_unchanged() {
    screen_render::GUARD.original().store(game_render as *mut c_void, Ordering::SeqCst);

    screen_render::detour(0x1000, 0x2000);

    assert_eq!(SEEN.lock().unwrap().clone(), [(0x1000, 0x2000, false)]);
}

#[test]
fn a_disabled_event_is_applied_after_the_original_and_clears_the_hover() {
    let runtime = RuntimeHarness::start(&[]);
    let collection = RuntimeHarness::collection("container_items");
    screen_render::GUARD.original().store(game_render as *mut c_void, Ordering::SeqCst);
    slot_hovered::GUARD.original().store(game_hovered as *mut c_void, Ordering::SeqCst);
    slot_hovered::detour(0x1000, collection, 7);
    assert!(runtime.is_on());
    assert!(runtime.is_tracking(0x1000));
    FakeTable::push_event(ModEvent::Disabled);

    screen_render::detour(0, 0);

    assert_eq!(SEEN.lock().unwrap().clone(), [(0, 0, true)]);
    assert!(!runtime.is_on());
    assert!(!runtime.is_tracking(0x1000));
}

#[test]
fn while_off_hovers_are_not_tracked_and_an_enabled_event_turns_the_mod_on_again() {
    let runtime = RuntimeHarness::start(&[]);
    let collection = RuntimeHarness::collection("container_items");
    screen_render::GUARD.original().store(game_render as *mut c_void, Ordering::SeqCst);
    slot_hovered::GUARD.original().store(game_hovered as *mut c_void, Ordering::SeqCst);
    FakeTable::push_event(ModEvent::Disabled);
    screen_render::detour(0, 0);

    slot_hovered::detour(0x1000, collection, 7);
    assert!(!runtime.is_tracking(0x1000));

    FakeTable::push_event(ModEvent::Enabled);
    screen_render::detour(0, 0);
    slot_hovered::detour(0x1000, collection, 7);

    assert!(runtime.is_on());
    assert!(runtime.is_tracking(0x1000));
}

#[test]
fn without_the_manager_the_mod_stays_off() {
    let runtime = PeekRuntime::install(std::path::Path::new(FakeTable::CONFIG_DIR));
    runtime.start(crate::support::FakeResolver::entries(&[]));
    let collection = RuntimeHarness::collection("container_items");
    screen_render::GUARD.original().store(game_render as *mut c_void, Ordering::SeqCst);
    slot_hovered::GUARD.original().store(game_hovered as *mut c_void, Ordering::SeqCst);

    screen_render::detour(0, 0);
    screen_render::detour(0, 0);
    slot_hovered::detour(0x1000, collection, 7);

    assert!(!runtime.is_on());
    assert!(!runtime.is_tracking(0x1000));
    assert_eq!(SEEN.lock().unwrap().len(), 2);
}
