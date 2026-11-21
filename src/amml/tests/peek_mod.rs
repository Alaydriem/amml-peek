use alaydriem_peek::PeekMod;
use amm_client::link::LinkUses;
use amm_client::test_support::FakeTable;
use amm_core::structs::event::ModEvent;
use amml_api::AmmlMod;
use amml_api::amml_abi::ModsLoaded;
use amml_api::testing::{FakeEntry, FakeGame, FakeItem};

use crate::support::{PeekRig, SCREEN};

const MANIFEST: &str = include_str!("../amml.hcl");

// a chest with a shulker box in slot 0: a stone stack, a pickaxe, and two bundles
fn box_entries() -> Vec<FakeEntry> {
    vec![
        PeekRig::item(0, "minecraft:stone", 64),
        FakeEntry {
            slot: 1,
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
        PeekRig::bundle(2, 16, vec![PeekRig::item(0, "minecraft:apple", 3), PeekRig::item(1, "minecraft:dirt", 1)]),
        PeekRig::bundle(3, 48, vec![PeekRig::item(0, "minecraft:gold_ingot", 9)]),
    ]
}

fn chest() -> amml_api::testing::FakeScreen {
    PeekRig::chest(vec![Some(PeekRig::shulker(box_entries())), None])
}

#[test]
fn enable_logs_the_features_and_the_link() {
    let rig = PeekRig::with_manager();

    rig.render(&chest());

    assert!(rig.logged("info features off: none"));
    assert!(rig.logged("info peek linked"));
}

#[test]
fn the_hovered_box_is_drawn_with_its_items_counts_and_glint() {
    let rig = PeekRig::with_manager();
    let screen = chest();
    rig.hover(0);

    rig.render(&screen);

    let draws = rig.game.draws();
    assert!(draws.iter().any(|draw| draw.starts_with("fill ")));
    assert!(draws.iter().any(|draw| draw.starts_with("outline ")));
    let items = rig.item_draws();
    assert!(items.iter().any(|draw| draw.starts_with("item minecraft:stone ")), "{items:?}");
    assert!(items.iter().any(|draw| draw.starts_with("item minecraft:diamond_pickaxe ")), "{items:?}");
    assert!(items.iter().any(|draw| draw.starts_with("item minecraft:apple ")), "{items:?}");
    assert!(draws.iter().any(|draw| draw.starts_with("text \"64\" ")), "{draws:?}");
    assert!(rig.logged("info peek hover screen=7 collection=container slot=0"));
    assert!(rig.logged("info peek box items=4 bundles=2"));
    assert!(rig.logged_count("info peek counts drawn=") == 1);
}

#[test]
fn the_glint_item_is_drawn_twice_and_the_second_draw_is_the_glint() {
    let rig = PeekRig::with_manager();
    rig.hover(0);

    rig.render(&chest());

    let pickaxe: Vec<String> =
        rig.item_draws().into_iter().filter(|draw| draw.starts_with("item minecraft:diamond_pickaxe ")).collect();
    assert_eq!(pickaxe.len(), 2, "{pickaxe:?}");
    assert_ne!(pickaxe[0], pickaxe[1]);
}

#[test]
fn the_box_is_read_once_for_a_key_and_again_when_the_key_changes() {
    let rig = PeekRig::with_manager();
    rig.hover(0);
    let screen = chest();

    rig.render(&screen);
    rig.render(&screen);
    rig.render(&screen);

    assert_eq!(rig.logged_count("info peek box "), 1);
    let mut counted = screen.clone();
    counted.slots.container[0] = Some(FakeItem { count: 2, ..PeekRig::shulker(box_entries()) });

    rig.render(&counted);

    assert_eq!(rig.logged_count("info peek box "), 2);
}

#[test]
fn the_old_item_copies_are_given_back_before_a_new_box_is_read() {
    let rig = PeekRig::with_manager();
    rig.hover(0);
    rig.render(&chest());
    assert_eq!(rig.game.live_item_draws(), 7);
    let mut other = chest();
    other.slots.container[0] = Some(PeekRig::shulker(vec![PeekRig::item(0, "minecraft:sand", 2)]));
    other.slots.container[0] = other.slots.container[0].map(|item| FakeItem { count: 2, ..item });

    rig.render(&other);

    assert_eq!(rig.game.pump_item_draws(), 0);
    assert_eq!(rig.game.live_item_draws(), 1);
}

#[test]
fn an_unhover_ends_the_panel_and_a_slot_with_no_box_draws_nothing() {
    let rig = PeekRig::with_manager();
    let screen = chest();
    rig.hover(0);
    rig.unhover(0);

    rig.render(&screen);

    assert!(rig.game.draws().is_empty());
    rig.hover(1);
    rig.render(&screen);
    assert!(rig.game.draws().is_empty());
}

#[test]
fn a_stack_that_is_no_shulker_box_draws_nothing() {
    let rig = PeekRig::with_manager();
    let screen = PeekRig::chest(vec![Some(FakeItem { type_name: "minecraft:stone", count: 5, ..FakeItem::EMPTY })]);
    rig.hover(0);

    rig.render(&screen);

    assert!(rig.game.draws().is_empty());
    assert_eq!(rig.logged_count("info peek box "), 0);
}

#[test]
fn an_empty_box_draws_its_grid_and_no_items() {
    let rig = PeekRig::with_manager();
    let screen = PeekRig::chest(vec![Some(FakeItem {
        type_name: "minecraft:white_shulker_box",
        count: 1,
        identity: Some(5),
        aux: Some(0),
        ..FakeItem::EMPTY
    })]);
    rig.hover(0);

    rig.render(&screen);

    assert!(rig.game.draws().iter().filter(|draw| draw.starts_with("fill ")).count() >= 28);
    assert!(rig.item_draws().is_empty());
    assert_eq!(rig.game.live_item_draws(), 0);
}

#[test]
fn a_view_that_is_not_a_container_screen_draws_nothing() {
    let rig = PeekRig::with_manager();
    let mut screen = chest();
    screen.kind = amml_api::ScreenKind::Other;
    rig.hover(0);

    rig.render(&screen);

    assert!(rig.game.draws().is_empty());
}

#[test]
fn a_view_with_no_pointer_or_no_size_draws_nothing() {
    let rig = PeekRig::with_manager();
    rig.hover(0);
    let mut screen = chest();
    screen.pointer = None;
    rig.render(&screen);
    let mut screen = chest();
    screen.size = None;

    rig.render(&screen);

    assert!(rig.game.draws().is_empty());
}

#[test]
fn the_screen_closing_ends_the_panel_and_is_logged() {
    let rig = PeekRig::with_manager();
    rig.hover(0);
    rig.render(&chest());
    assert_eq!(rig.game.live_item_draws(), 7);

    rig.game.raise_screen_closing(SCREEN);

    assert!(rig.logged("info peek screen closed screen=7"));
    assert_eq!(rig.game.pump_item_draws(), 0);
    assert_eq!(rig.game.live_item_draws(), 0);
    let before = rig.game.draws().len();
    rig.render(&chest());
    assert_eq!(rig.game.draws().len(), before);
}

#[test]
fn shift_and_wheel_pick_a_bundle_and_the_screen_does_not_scroll() {
    let rig = PeekRig::with_manager();
    rig.hover(0);
    rig.render(&chest());
    rig.shift(true);

    let taken = rig.game.raise_scroll(Some(SCREEN), -1);

    assert!(taken);
    assert!(rig.logged("info peek bundle selected index=2"));
    rig.render(&chest());
    assert!(rig.item_draws().iter().any(|draw| draw.starts_with("item minecraft:gold_ingot ")));
    assert!(rig.game.raise_scroll(Some(SCREEN), 1));
    assert!(rig.logged("info peek bundle selected index=1"));
}

#[test]
fn a_wheel_step_without_shift_or_with_one_bundle_or_on_another_screen_goes_to_the_game() {
    let rig = PeekRig::with_manager();
    rig.hover(0);
    rig.render(&chest());

    assert!(!rig.game.raise_scroll(Some(SCREEN), -1));
    rig.shift(true);
    assert!(!rig.game.raise_scroll(Some(amml_api::ScreenId(8)), -1));
    assert!(!rig.game.raise_scroll(None, -1));
    assert!(!rig.game.raise_scroll(Some(SCREEN), 0));
    assert_eq!(rig.logged_count("info peek bundle selected"), 0);

    let one = PeekRig::chest(vec![Some(PeekRig::shulker(vec![PeekRig::bundle(
        0,
        8,
        vec![PeekRig::item(0, "minecraft:apple", 1)],
    )]))]);
    rig.game.raise_screen_closing(SCREEN);
    rig.hover(0);
    rig.render(&one);
    assert!(!rig.game.raise_scroll(Some(SCREEN), -1));
}

#[test]
fn the_contents_lines_of_the_hover_text_are_hidden_while_the_preview_is_on() {
    let rig = PeekRig::with_manager();
    rig.render(&chest());

    assert_eq!(rig.game.raise_hovertext(), 0);
    assert_eq!(rig.game.raise_hovertext(), 0);

    assert_eq!(rig.logged_count("info peek hovertext contents_lines=0"), 1);
}

#[test]
fn the_manager_turns_the_preview_off_and_on() {
    let rig = PeekRig::with_manager();
    let screen = chest();
    rig.hover(0);
    rig.render(&screen);
    assert!(!rig.game.draws().is_empty());

    FakeTable::push_event(ModEvent::Disabled);
    rig.render(&screen);
    let drawn = rig.game.draws().len();
    rig.render(&screen);

    assert_eq!(rig.game.draws().len(), drawn);
    assert_eq!(rig.game.raise_hovertext(), 1);
    assert_eq!(rig.game.live_item_draws(), 0);

    FakeTable::push_event(ModEvent::Enabled);
    rig.render(&screen);
    rig.hover(0);
    rig.render(&screen);

    assert!(rig.game.draws().len() > drawn);
    assert_eq!(rig.game.raise_hovertext(), 0);
}

#[test]
fn without_the_manager_the_preview_stays_off_and_one_error_is_logged() {
    let rig = PeekRig::without_manager();
    rig.hover(0);

    rig.game.raise(&mut ModsLoaded { header: FakeGame::header::<ModsLoaded>() });
    rig.render(&chest());

    assert!(rig.game.draws().is_empty());
    assert_eq!(rig.game.raise_hovertext(), 1);
    assert!(rig.logged("error @alaydriem/peek needs @amml/modmanager; the preview stays off"));
}

#[test]
fn a_manager_that_goes_away_turns_the_preview_off() {
    let rig = PeekRig::with_manager();
    rig.hover(0);
    rig.render(&chest());
    assert!(!rig.game.draws().is_empty());

    FakeTable::gone(&rig.game);
    rig.render(&chest());
    let drawn = rig.game.draws().len();
    rig.render(&chest());

    assert_eq!(rig.game.draws().len(), drawn);
    assert_eq!(rig.game.raise_hovertext(), 1);
}

#[test]
fn item_copies_of_a_left_world_are_made_again() {
    let rig = PeekRig::with_manager();
    rig.hover(0);
    let screen = chest();
    rig.render(&screen);
    rig.game.leave_item_world();
    let before = rig.item_draws().len();

    rig.render(&screen);
    let after_stale = rig.item_draws().len();
    rig.render(&screen);

    assert_eq!(after_stale, before);
    assert!(rig.item_draws().len() > after_stale);
    assert_eq!(rig.logged_count("info peek box "), 2);
}

#[test]
fn disable_removes_the_listeners_and_gives_back_the_copies_and_the_link() {
    let rig = PeekRig::with_manager();
    rig.hover(0);
    rig.render(&chest());
    assert_eq!(rig.game.live_item_draws(), 7);

    rig.game.disable::<PeekMod>();

    assert_eq!(rig.game.live_item_draws(), 0);
    assert!(FakeTable::calls().contains(&"unregister"));
    let before = rig.game.draws().len();
    rig.hover(0);
    rig.render(&chest());
    assert_eq!(rig.game.draws().len(), before);
    assert_eq!(rig.game.raise_hovertext(), 1);
}

#[test]
fn a_refused_required_listen_is_logged_and_fails_enable() {
    let game = PeekRig::offered();
    game.refuse_listen::<amml_api::amml_abi::ScreenRender>();
    FakeTable::provide(&game.ctx()).unwrap();
    game.load::<PeekMod>().unwrap();

    assert!(game.enable::<PeekMod>().is_err());

    assert!(game.logs().iter().any(|line| line.starts_with("error peek off error=")));
}

#[test]
fn the_manifest_names_the_mod_and_lists_no_entries() {
    assert!(MANIFEST.contains("namespace    = \"alaydriem\""));
    assert!(MANIFEST.contains("name         = \"peek\""));
    assert!(MANIFEST.contains("api_version  = 3"));
    assert!(!MANIFEST.contains("entries"));
}

#[test]
fn link_uses_are_complete() {
    assert_eq!(LinkUses::check(&PeekMod::USES), Ok(()));
}

#[test]
fn every_row_the_mod_uses_is_required_except_the_rows_of_the_link() {
    let hcl = PeekMod::USES.to_hcl();

    assert_eq!(hcl.matches("required = false").count(), LinkUses::OPTIONAL.len() - 1);
    assert!(hcl.contains("required = true"));
}

#[test]
fn the_manifest_uses_block_is_the_uses_of_the_mod() {
    let manifest = MANIFEST.replace("\r\n", "\n");

    assert_eq!(manifest.matches(&PeekMod::USES.to_hcl()).count(), 1);
    assert_eq!(PeekMod::USES.write_into(&manifest), manifest);
}
