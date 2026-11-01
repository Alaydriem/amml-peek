use std::path::Path;

use alaydriem_peek::services::ManagerSession;
use alaydriem_peek::types::ManagerDelta;
use amm_client::test_support::FakeTable;
use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::{KeybindPhase, ModEvent};

fn session() -> ManagerSession {
    ManagerSession::connect(FakeTable::table(), Path::new(FakeTable::CONFIG_DIR)).unwrap()
}

#[test]
fn connect_registers_the_descriptor_with_no_frame_callback() {
    let _session = session();

    let descriptor = FakeTable::with(|state| state.descriptor.clone()).unwrap();
    let callbacks = FakeTable::with(|state| state.callbacks).unwrap();

    assert_eq!(descriptor, ManagerSession::DESCRIPTOR);
    assert!(callbacks.frame.is_none());
}

#[test]
fn no_event_changes_nothing() {
    let mut session = session();

    assert_eq!(session.drain(), ManagerDelta { on: None, disabled: false });
}

#[test]
fn disabled_turns_the_mod_off_and_enabled_turns_it_on() {
    let mut session = session();

    FakeTable::push_event(ModEvent::Disabled);
    assert_eq!(session.drain(), ManagerDelta { on: Some(false), disabled: true });

    FakeTable::push_event(ModEvent::Enabled);
    assert_eq!(session.drain(), ManagerDelta { on: Some(true), disabled: false });
}

#[test]
fn a_disable_followed_by_an_enable_in_one_drain_still_reports_the_disable() {
    let mut session = session();

    FakeTable::push_event(ModEvent::Disabled);
    FakeTable::push_event(ModEvent::Enabled);

    assert_eq!(session.drain(), ManagerDelta { on: Some(true), disabled: true });
}

#[test]
fn other_events_are_ignored() {
    let mut session = session();

    FakeTable::push_event(ModEvent::Keybind { action: 0, phase: KeybindPhase::Pressed });
    FakeTable::push_event(ModEvent::Resumed(Capability::WorldDraw));
    FakeTable::push_event(ModEvent::LocaleChanged);

    assert_eq!(session.drain(), ManagerDelta { on: None, disabled: false });
}
