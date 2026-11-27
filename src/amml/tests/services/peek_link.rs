use alaydriem_peek::services::PeekLink;
use alaydriem_peek::types::ManagerDelta;
use amm_client::link::LinkState;
use amm_client::test_support::FakeTable;
use amm_core::services::DescriptorChecker;
use amm_core::structs::event::ModEvent;
use amml_api::testing::FakeGame;

// a link connected to the fake manager of this thread; the game stays alive with it
fn connected() -> (FakeGame, PeekLink) {
    let game = FakeTable::game();
    FakeTable::provide(&game.ctx()).unwrap();
    let link = PeekLink::start(&FakeTable::ctx_of(&game, PeekLink::DESCRIPTOR)).unwrap();
    (game, link)
}

#[test]
fn the_link_connects_when_the_manager_is_provided() {
    let (_game, link) = connected();

    assert_eq!(link.state(), LinkState::Connected);
}

#[test]
fn the_link_waits_while_there_is_no_manager() {
    let game = FakeTable::game();

    let link = PeekLink::start(&FakeTable::ctx_of(&game, PeekLink::DESCRIPTOR)).unwrap();

    assert_eq!(link.state(), LinkState::Waiting);
}

#[test]
fn an_enable_and_a_disable_give_the_on_state_and_a_disable_is_noted_also_when_an_enable_follows() {
    let (_game, mut link) = connected();

    FakeTable::push_event(ModEvent::Enabled);
    assert_eq!(link.drain(), ManagerDelta { on: Some(true), disabled: false });

    FakeTable::push_event(ModEvent::Disabled);
    assert_eq!(link.drain(), ManagerDelta { on: Some(false), disabled: true });

    FakeTable::push_event(ModEvent::Disabled);
    FakeTable::push_event(ModEvent::Enabled);
    assert_eq!(link.drain(), ManagerDelta { on: Some(true), disabled: true });
    assert_eq!(link.drain(), ManagerDelta::default());
}

#[test]
fn the_manager_going_away_turns_the_preview_off() {
    let (game, mut link) = connected();

    FakeTable::gone(&game);

    let delta = link.drain();
    assert_eq!(delta.on, Some(false));
    assert!(delta.disabled);
}

#[test]
fn the_descriptor_passes_the_manager_checker_with_no_claim_keybind_or_setting() {
    let descriptor = DescriptorChecker::new(128, 256).parse(PeekLink::DESCRIPTOR).unwrap();

    assert_eq!(descriptor.id(), "@alaydriem/peek");
    assert_eq!(descriptor.display_name, "Shulker Peek");
    assert_eq!(descriptor.version, env!("CARGO_PKG_VERSION"));
    assert!(descriptor.claims.is_empty());
    assert!(descriptor.keybinds.is_empty());
    assert!(descriptor.settings.is_empty());
}
