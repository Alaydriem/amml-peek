use std::cell::Cell;
use std::rc::Rc;

use alaydriem_peek::services::PreviewCache;
use alaydriem_peek::types::PreviewKey;

use crate::support::{FakeBox, FakeEntry, LogCapture, TrackedStack};

const SKIPPED: &str = "shulker box entries skipped";

fn key(stack: usize, user_data: usize, count: u8) -> PreviewKey {
    PreviewKey { stack, user_data, count }
}

fn drops() -> Rc<Cell<usize>> {
    Rc::new(Cell::new(0))
}

fn name(cache: &PreviewCache<TrackedStack>, slot: usize) -> Option<&str> {
    cache.item(slot).map(|item| item.stack.name.as_str())
}

#[test]
fn bad_entries_are_skipped_the_rest_is_placed_and_the_skips_are_logged_once() {
    LogCapture::install();
    let drops = drops();
    let source = FakeBox::new(
        &drops,
        vec![
            FakeEntry::item(Some(0), "stone"),
            FakeEntry::item(None, "dirt"),
            FakeEntry::item(Some(40), "sand"),
            FakeEntry::unknown(Some(2)),
        ],
    );
    let mut cache = PreviewCache::new();

    cache.sync(key(1, 2, 3), Some(&source));

    assert_eq!(name(&cache, 0), Some("stone"));
    assert!((1..27).all(|slot| cache.item(slot).is_none()));
    assert_eq!(cache.skipped(), 3);
    assert_eq!(source.built(), [0, 3]);
    assert_eq!(LogCapture::count(SKIPPED), 1);

    cache.sync(key(1, 2, 3), Some(&source));

    assert_eq!(LogCapture::count(SKIPPED), 1);
    assert_eq!(source.built(), [0, 3]);
}

#[test]
fn a_negative_slot_is_skipped() {
    let drops = drops();
    let source = FakeBox::new(&drops, vec![FakeEntry::item(Some(-1), "stone"), FakeEntry::item(Some(26), "dirt")]);
    let mut cache = PreviewCache::new();

    cache.sync(key(1, 0, 1), Some(&source));

    assert_eq!(name(&cache, 26), Some("dirt"));
    assert_eq!(cache.skipped(), 1);
}

#[test]
fn a_later_entry_for_the_same_slot_wins_and_the_earlier_one_is_not_built() {
    let drops = drops();
    let source = FakeBox::new(&drops, vec![FakeEntry::item(Some(4), "stone"), FakeEntry::item(Some(4), "dirt")]);
    let mut cache = PreviewCache::new();

    cache.sync(key(1, 0, 1), Some(&source));

    assert_eq!(name(&cache, 4), Some("dirt"));
    assert_eq!(source.built(), [1]);
}

#[test]
fn the_same_key_builds_once_and_a_new_key_drops_the_old_stacks_and_builds_again() {
    let drops = drops();
    let source = FakeBox::new(&drops, vec![FakeEntry::item(Some(0), "stone")]);
    let mut cache = PreviewCache::new();

    cache.sync(key(1, 2, 3), Some(&source));
    cache.sync(key(1, 2, 3), Some(&source));

    assert_eq!(source.built(), [0]);
    assert_eq!(drops.get(), 0);

    cache.sync(key(1, 2, 4), Some(&source));

    assert_eq!(drops.get(), 1);
    assert_eq!(source.built(), [0, 0]);
    assert_eq!(cache.key(), Some(key(1, 2, 4)));
}

#[test]
fn each_part_of_the_key_makes_a_new_key() {
    let drops = drops();
    let source = FakeBox::new(&drops, vec![FakeEntry::item(Some(0), "stone")]);
    let mut cache = PreviewCache::new();

    cache.sync(key(1, 2, 3), Some(&source));
    cache.sync(key(9, 2, 3), Some(&source));
    cache.sync(key(9, 8, 3), Some(&source));
    cache.sync(key(9, 8, 7), Some(&source));

    assert_eq!(source.built().len(), 4);
    assert_eq!(drops.get(), 3);
}

#[test]
fn clear_drops_every_stack_and_forgets_the_key() {
    let drops = drops();
    let source = FakeBox::new(&drops, vec![FakeEntry::item(Some(0), "stone"), FakeEntry::item(Some(1), "dirt")]);
    let mut cache = PreviewCache::new();
    cache.sync(key(1, 2, 3), Some(&source));

    cache.clear();

    assert_eq!(drops.get(), 2);
    assert_eq!(cache.key(), None);
    assert!((0..27).all(|slot| cache.item(slot).is_none()));
}

#[test]
fn a_box_with_no_contents_has_27_empty_slots_and_no_skips() {
    let mut cache: PreviewCache<TrackedStack> = PreviewCache::new();

    cache.sync(key(1, 0, 1), None::<&FakeBox>);

    assert_eq!(cache.key(), Some(key(1, 0, 1)));
    assert_eq!(cache.skipped(), 0);
    assert!((0..27).all(|slot| cache.item(slot).is_none()));
}

#[test]
fn a_slot_past_the_grid_has_no_item() {
    let cache: PreviewCache<TrackedStack> = PreviewCache::new();

    assert!(cache.item(27).is_none());
    assert!(cache.item(usize::MAX).is_none());
}

fn bundle_of(entries: Vec<FakeEntry>) -> FakeEntry {
    FakeEntry::item(Some(0), "bundle").bundle(Some(32), Some(entries))
}

fn strip_names(cache: &PreviewCache<TrackedStack>) -> Vec<Option<&str>> {
    (0..cache.strip_len()).map(|position| cache.strip_item(position).map(|item| item.stack.name.as_str())).collect()
}

#[test]
fn bundles_are_noted_in_slot_order_with_their_weight_and_the_grid_is_unchanged() {
    let drops = drops();
    let source = FakeBox::new(
        &drops,
        vec![
            FakeEntry::item(Some(20), "bundle").bundle(None, None),
            FakeEntry::item(Some(3), "stone"),
            FakeEntry::item(Some(5), "bundle").bundle(Some(12), None),
        ],
    );
    let mut cache = PreviewCache::new();

    let changed = cache.sync(key(1, 2, 3), Some(&source));

    assert!(changed);
    let bundles: Vec<(u8, Option<i32>)> = cache.bundles().iter().map(|bundle| (bundle.slot, bundle.weight)).collect();
    assert_eq!(bundles, [(5, Some(12)), (20, None)]);
    assert_eq!(name(&cache, 3), Some("stone"));
    assert_eq!(name(&cache, 5), Some("bundle"));
    assert!(!cache.sync(key(1, 2, 3), Some(&source)));
}

#[test]
fn the_strip_holds_the_selected_bundle_in_slot_order_and_bad_slots_after_the_good_ones() {
    let drops = drops();
    let inside = vec![
        FakeEntry::item(None, "late"),
        FakeEntry::item(Some(4), "four"),
        FakeEntry::item(Some(-2), "negative"),
        FakeEntry::item(Some(1), "one"),
    ];
    let source = FakeBox::new(&drops, vec![bundle_of(inside)]);
    let mut cache = PreviewCache::new();
    cache.sync(key(1, 2, 3), Some(&source));

    cache.sync_strip(Some(&source), Some(0));

    assert_eq!(strip_names(&cache), [Some("one"), Some("four"), Some("late"), Some("negative")]);
}

#[test]
fn a_bundle_with_a_missing_list_gives_an_empty_strip_and_the_grid_stays() {
    let drops = drops();
    let source = FakeBox::new(
        &drops,
        vec![FakeEntry::item(Some(0), "bundle").bundle(Some(1), None), FakeEntry::item(Some(1), "stone")],
    );
    let mut cache = PreviewCache::new();
    cache.sync(key(1, 2, 3), Some(&source));

    cache.sync_strip(Some(&source), Some(0));

    assert_eq!(cache.strip_len(), 0);
    assert!(cache.strip_item(0).is_none());
    assert_eq!(name(&cache, 1), Some("stone"));
    assert_eq!(cache.skipped(), 0);
}

#[test]
fn entries_the_game_cannot_build_are_left_out_and_the_rest_close_up() {
    LogCapture::install();
    let drops = drops();
    let inside = vec![FakeEntry::item(Some(0), "one"), FakeEntry::unknown(Some(1)), FakeEntry::item(Some(2), "three")];
    let source = FakeBox::new(&drops, vec![bundle_of(inside)]);
    let mut cache = PreviewCache::new();
    cache.sync(key(1, 2, 3), Some(&source));

    cache.sync_strip(Some(&source), Some(0));
    cache.sync_strip(Some(&source), Some(0));

    assert_eq!(strip_names(&cache), [Some("one"), Some("three")]);
    assert_eq!(LogCapture::count("bundle entries skipped"), 0);
}

#[test]
fn more_than_64_entries_build_64() {
    let drops = drops();
    let inside: Vec<FakeEntry> = (0..100).map(|slot| FakeEntry::item(Some(slot), "item")).collect();
    let source = FakeBox::new(&drops, vec![bundle_of(inside)]);
    let mut cache = PreviewCache::new();
    cache.sync(key(1, 2, 3), Some(&source));

    cache.sync_strip(Some(&source), Some(0));

    assert_eq!(cache.strip_len(), 64);
    assert!(cache.strip_item(63).is_some());
    assert!(cache.strip_item(64).is_none());
    cache.clear();
    assert_eq!(drops.get(), 1 + 64);
}

#[test]
fn a_selection_change_drops_the_old_strip_at_once_and_builds_the_new_one() {
    let drops = drops();
    let first = vec![FakeEntry::item(Some(0), "a"), FakeEntry::item(Some(1), "b")];
    let second = vec![FakeEntry::item(Some(0), "c")];
    let source = FakeBox::new(
        &drops,
        vec![
            FakeEntry::item(Some(0), "bundle").bundle(Some(1), Some(first)),
            FakeEntry::item(Some(1), "bundle").bundle(Some(1), Some(second)),
        ],
    );
    let mut cache = PreviewCache::new();
    cache.sync(key(1, 2, 3), Some(&source));
    cache.sync_strip(Some(&source), Some(0));
    assert_eq!(drops.get(), 0);

    cache.sync_strip(Some(&source), Some(0));
    assert_eq!(drops.get(), 0);

    cache.drop_strip();
    assert_eq!(drops.get(), 2);
    assert_eq!(cache.strip_len(), 0);

    cache.sync_strip(Some(&source), Some(1));
    assert_eq!(strip_names(&cache), [Some("c")]);
    cache.sync_strip(Some(&source), Some(0));
    assert_eq!(drops.get(), 3);
    assert_eq!(strip_names(&cache), [Some("a"), Some("b")]);
}

#[test]
fn a_box_change_drops_the_box_and_the_strip_stacks() {
    let drops = drops();
    let inside = vec![FakeEntry::item(Some(0), "a"), FakeEntry::item(Some(1), "b")];
    let source = FakeBox::new(&drops, vec![bundle_of(inside)]);
    let other = FakeBox::new(&drops, vec![FakeEntry::item(Some(0), "stone")]);
    let mut cache = PreviewCache::new();
    cache.sync(key(1, 2, 3), Some(&source));
    cache.sync_strip(Some(&source), Some(0));

    let changed = cache.sync(key(9, 2, 3), Some(&other));

    assert!(changed);
    assert_eq!(drops.get(), 3);
    assert_eq!(cache.strip_len(), 0);
    assert!(cache.bundles().is_empty());
    assert_eq!(name(&cache, 0), Some("stone"));
}

#[test]
fn no_selection_builds_no_strip() {
    let drops = drops();
    let source = FakeBox::new(&drops, vec![bundle_of(vec![FakeEntry::item(Some(0), "a")])]);
    let mut cache = PreviewCache::new();
    cache.sync(key(1, 2, 3), Some(&source));

    cache.sync_strip(Some(&source), None);

    assert_eq!(cache.strip_len(), 0);
}
