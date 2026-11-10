use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use alaydriem_peek::detours::{screen_render, shulker_hovertext};
use amm_client::test_support::FakeTable;
use amm_core::structs::event::ModEvent;
use amml_game::EntryId;

use crate::support::RuntimeHarness;

// which function ran, with this, the stack, the level, the text and the flag
static CALLS: Mutex<Vec<(&'static str, [usize; 5])>> = Mutex::new(Vec::new());

extern "C" fn game_override(this: usize, stack: usize, level: usize, text: usize, flag: usize) {
    CALLS.lock().unwrap().push(("override", [this, stack, level, text, flag]));
}

extern "C" fn game_base(this: usize, stack: usize, level: usize, text: usize, flag: usize) {
    CALLS.lock().unwrap().push(("base", [this, stack, level, text, flag]));
}

extern "C" fn game_render(_: usize, _: usize) {}

fn base_entry() -> [(EntryId, u64); 1] {
    [(EntryId::ITEM_APPEND_FORMATTED_HOVERTEXT, game_base as *const () as usize as u64)]
}

fn call() {
    shulker_hovertext::detour(1, 2, 3, 4, 5);
}

fn calls() -> Vec<(&'static str, [usize; 5])> {
    std::mem::take(&mut *CALLS.lock().unwrap())
}

#[test]
fn without_a_runtime_the_original_runs_with_the_same_arguments() {
    shulker_hovertext::GUARD.original().store(game_override as *mut c_void, Ordering::SeqCst);

    call();

    assert_eq!(calls(), [("override", [1, 2, 3, 4, 5])]);
}

#[test]
fn while_on_the_base_function_runs_instead_of_the_override_with_the_same_arguments() {
    RuntimeHarness::start(&base_entry());
    shulker_hovertext::GUARD.original().store(game_override as *mut c_void, Ordering::SeqCst);

    call();

    assert_eq!(calls(), [("base", [1, 2, 3, 4, 5])]);
}

#[test]
fn after_a_disabled_event_the_original_runs_and_after_an_enabled_event_the_base_runs_again() {
    RuntimeHarness::start(&base_entry());
    shulker_hovertext::GUARD.original().store(game_override as *mut c_void, Ordering::SeqCst);
    screen_render::GUARD.original().store(game_render as *mut c_void, Ordering::SeqCst);

    FakeTable::push_event(ModEvent::Disabled);
    screen_render::detour(0, 0);
    call();
    assert_eq!(calls(), [("override", [1, 2, 3, 4, 5])]);

    FakeTable::push_event(ModEvent::Enabled);
    screen_render::detour(0, 0);
    call();
    assert_eq!(calls(), [("base", [1, 2, 3, 4, 5])]);
}

#[test]
fn after_stop_the_original_runs() {
    let runtime = RuntimeHarness::start(&base_entry());
    shulker_hovertext::GUARD.original().store(game_override as *mut c_void, Ordering::SeqCst);

    runtime.stop();
    call();

    assert_eq!(calls(), [("override", [1, 2, 3, 4, 5])]);
}
