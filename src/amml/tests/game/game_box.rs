use alaydriem_peek::game::GameBox;
use alaydriem_peek::traits::BoxSource;
use amml_api::ScreenRenderExt;
use amml_api::testing::{FakeEntry, FakeItem};

use crate::support::{PeekRig, ScreenProbe};

fn entries() -> Vec<FakeEntry> {
    vec![
        PeekRig::item(0, "minecraft:stone", 64),
        FakeEntry {
            slot: 5,
            item: FakeItem {
                type_name: "minecraft:diamond_pickaxe",
                count: 1,
                damage: 100,
                max_damage: 1561,
                is_glint: Some(true),
                ..FakeItem::EMPTY
            },
            bundle: None,
            bundle_weight: None,
        },
        PeekRig::bundle(9, 16, vec![PeekRig::item(0, "minecraft:apple", 3), PeekRig::item(4, "minecraft:dirt", 1)]),
    ]
}

// the box of slot 0 of a chest, read inside a render call, as `body` sees it
fn with_box<R: Send + 'static>(body: impl FnOnce(GameBox) -> R + Send + 'static) -> R {
    let game = PeekRig::offered();
    game.open_gate();
    let screen = PeekRig::chest(vec![Some(PeekRig::shulker(entries()))]);
    ScreenProbe::run(&game, &screen, move |event| {
        let container = event.container_screen().unwrap();
        let stack = container.visual_item(PeekRig::container(0)).unwrap();
        body(GameBox::new(stack.container_contents().unwrap()))
    })
}

#[test]
fn an_entry_carries_its_slot_count_glint_and_durability() {
    let facts = with_box(|source| {
        let stone = source.build(0).unwrap();
        let pickaxe = source.build(1).unwrap();
        (
            source.len(),
            [source.slot(0), source.slot(1), source.slot(2), source.slot(3)],
            (stone.count_text, stone.glint, stone.bar.is_some()),
            (pickaxe.count_text, pickaxe.glint, pickaxe.bar.is_some()),
        )
    });

    assert_eq!(facts.0, 3);
    assert_eq!(facts.1, [Some(0), Some(5), Some(9), None]);
    assert_eq!(facts.2, (Some("64".to_string()), false, false));
    assert_eq!(facts.3, (None, true, true));
}

#[test]
fn only_a_bundle_has_a_weight_and_a_bundle_entry_gives_its_own_box() {
    let facts = with_box(|source| {
        let bundle = source.build(2).unwrap();
        let inner = source.bundle_contents(2).unwrap();
        (
            (bundle.bundle, bundle.weight),
            source.build(0).map(|item| (item.bundle, item.weight)),
            inner.len(),
            [inner.slot(0), inner.slot(1)],
            source.bundle_contents(0).is_none(),
            source.bundle_contents(7).is_none(),
        )
    });

    assert_eq!(facts.0, (true, Some(16)));
    assert_eq!(facts.1, Some((false, None)));
    assert_eq!(facts.2, 2);
    assert_eq!(facts.3, [Some(0), Some(4)]);
    assert!(facts.4);
    assert!(facts.5);
}

#[test]
fn an_entry_past_the_end_builds_nothing() {
    assert!(with_box(|source| source.build(3).is_none()));
}

#[test]
fn the_draw_handles_live_as_long_as_the_box_and_the_items_made_from_it() {
    let game = PeekRig::offered();
    game.open_gate();
    let screen = PeekRig::chest(vec![Some(PeekRig::shulker(entries()))]);
    let kept = ScreenProbe::run(&game, &screen, |event| {
        let container = event.container_screen().unwrap();
        let stack = container.visual_item(PeekRig::container(0)).unwrap();
        let source = GameBox::new(stack.container_contents().unwrap());
        source.build(0).unwrap()
    });

    game.pump_item_draws();
    assert_eq!(game.live_item_draws(), 1);
    drop(kept);
    game.pump_item_draws();
    assert_eq!(game.live_item_draws(), 0);
}

#[test]
fn a_box_of_27_full_bundles_reads_in_full() {
    let game = PeekRig::offered();
    game.open_gate();
    let bundles = (0..27u8)
        .map(|slot| {
            PeekRig::bundle(slot, 64, (0..64u8).map(|inner| PeekRig::item(inner, "minecraft:dirt", 1)).collect())
        })
        .collect::<Vec<_>>();
    let screen = PeekRig::chest(vec![Some(PeekRig::shulker(bundles))]);

    let facts = ScreenProbe::run(&game, &screen, move |event| {
        let container = event.container_screen().unwrap();
        let stack = container.visual_item(PeekRig::container(0)).unwrap();
        let source = GameBox::new(stack.container_contents().unwrap());
        let inner: Vec<Option<usize>> =
            (0..source.len()).map(|index| source.bundle_contents(index).map(|b| b.len())).collect();
        (source.len(), inner)
    });

    assert_eq!(facts.0, 27);
    assert!(facts.1.iter().all(|len| *len == Some(64)));
}
