use alaydriem_peek::game::GameBoxResolver;
use alaydriem_peek::traits::{BoxResolver, BoxSource};
use alaydriem_peek::types::PreviewKey;
use amml_api::ScreenRenderExt;
use amml_api::testing::{FakeItem, FakeScreen};
use peek_core::structs::{HoveredSlot, SlotCollection};

use crate::support::{PeekRig, SCREEN, ScreenProbe};

fn slot(index: i32) -> HoveredSlot {
    HoveredSlot { screen: SCREEN.0, collection: SlotCollection::Container, index }
}

fn shulker(count: u8, aux: i16, identity: Option<u64>) -> FakeItem {
    FakeItem { count, aux: Some(aux), identity, ..PeekRig::shulker(vec![PeekRig::item(0, "minecraft:stone", 2)]) }
}

// runs `body` with the resolver of the container screen of `screen`
fn with_resolver<R: Send + 'static>(
    screen: FakeScreen,
    body: impl FnOnce(&GameBoxResolver<'_, '_>) -> R + Send + 'static,
) -> R {
    let game = PeekRig::offered();
    game.open_gate();
    ScreenProbe::run(&game, &screen, move |event| {
        let container = event.container_screen().unwrap();
        body(&GameBoxResolver::new(&container))
    })
}

#[test]
fn an_empty_slot_has_no_key_and_a_slot_the_api_does_not_serve_has_none_either() {
    let screen = PeekRig::chest(vec![None, Some(shulker(1, 0, None))]);

    let keys = with_resolver(screen, |resolver| {
        let other = HoveredSlot { collection: SlotCollection::Other, ..slot(1) };
        (resolver.key(&slot(0)).is_none(), resolver.key(&other).is_none(), resolver.key(&slot(9)).is_none())
    });

    assert_eq!(keys, (true, true, true));
}

#[test]
fn the_key_changes_with_the_slot_the_identity_the_aux_and_the_count() {
    let screen = PeekRig::chest(vec![
        Some(shulker(1, 0, None)),
        Some(shulker(1, 0, None)),
        Some(shulker(2, 0, None)),
        Some(shulker(1, 3, None)),
        Some(shulker(1, 0, Some(8))),
    ]);

    let keys =
        with_resolver(screen, |resolver| (0..5).map(|index| resolver.key(&slot(index)).unwrap()).collect::<Vec<_>>());

    for other in 1..5 {
        assert_ne!(keys[0], keys[other], "slot {other}");
    }
    assert_eq!(keys[0].identity, None);
    assert_eq!(keys[0].screen, SCREEN);
}

#[test]
fn the_same_stack_gives_the_same_key_in_two_frames() {
    let screen = PeekRig::chest(vec![Some(shulker(1, 0, None))]);

    let same = with_resolver(screen, |resolver| resolver.key(&slot(0)) == resolver.key(&slot(0)));

    assert!(same);
}

#[test]
fn a_shulker_box_loads_its_contents_and_an_empty_one_loads_none() {
    let empty = FakeItem { contents: None, ..shulker(1, 0, Some(4)) };
    let screen = PeekRig::chest(vec![Some(shulker(1, 0, None)), Some(empty)]);

    let loaded = with_resolver(screen, |resolver| {
        let full = resolver.load(&slot(0)).unwrap();
        let none = resolver.load(&slot(1)).unwrap();
        (full.contents.map(|contents| contents.len()), none.contents.is_none())
    });

    assert_eq!(loaded, (Some(1), true));
}

#[test]
fn a_stack_that_is_no_shulker_box_and_an_empty_slot_load_nothing() {
    let stone = FakeItem { type_name: "minecraft:stone", count: 3, ..FakeItem::EMPTY };
    let screen = PeekRig::chest(vec![Some(stone), None]);

    let loaded =
        with_resolver(screen, |resolver| (resolver.load(&slot(0)).is_none(), resolver.load(&slot(1)).is_none()));

    assert_eq!(loaded, (true, true));
}

#[test]
fn a_swapped_box_of_the_same_type_and_count_has_another_key_through_the_change_stamp() {
    let stamped = |stamp: Option<u64>| FakeItem { change_stamp: stamp, ..shulker(1, 0, None) };
    let screen = PeekRig::chest(vec![Some(stamped(Some(1))), Some(stamped(Some(1))), Some(stamped(Some(2)))]);

    let keys =
        with_resolver(screen, |resolver| (0..3).map(|index| resolver.key(&slot(index)).unwrap()).collect::<Vec<_>>());

    assert_eq!(keys[0].change_stamp, Some(1));
    assert_eq!(PreviewKey { slot: keys[0].slot, ..keys[1] }, keys[0]);
    assert_eq!(keys[2].change_stamp, Some(2));
    assert_ne!(PreviewKey { slot: keys[0].slot, ..keys[2] }, keys[0]);
}
