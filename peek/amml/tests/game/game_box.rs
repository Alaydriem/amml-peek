use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use alaydriem_peek::game::GameBox;
use alaydriem_peek::services::PreviewCache;
use alaydriem_peek::types::PreviewKey;
use amml_game::{CompoundTagRef, EntryId, GameEntries, HashedStringLayout};
use amml_msvc::MsvcString;

use crate::support::{FakeCompound, FakeResolver, FakeValue, LogCapture, TestHeap};

// the offset of `Item::mFullName` in the fake items
const FULL_NAME: usize = 0x20;

static STACK_VTABLE: AtomicUsize = AtomicUsize::new(0);
static ITEM_COUNTER: AtomicUsize = AtomicUsize::new(0);
static BUNDLE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static UNKNOWN_TAG: AtomicUsize = AtomicUsize::new(0);
static DTOR_CALLS: AtomicUsize = AtomicUsize::new(0);
// the entry tags that are bundles, each with the user data its stack gets (0 for none)
static BUNDLES: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());

// builds a stack of count 5 in the hidden buffer, as fromTag does, except for the tag of an item the game does not
// know, which gets the empty stack. A tag registered in `BUNDLES` gives a bundle with its user data
extern "C" fn from_tag(buffer: *mut c_void, tag: *const c_void) -> *mut c_void {
    let known = tag as usize != UNKNOWN_TAG.load(Ordering::SeqCst);
    let bundle = BUNDLES.lock().unwrap().iter().find(|(entry, _)| *entry == tag as usize).map(|(_, data)| *data);
    let counter =
        if bundle.is_some() { BUNDLE_COUNTER.load(Ordering::SeqCst) } else { ITEM_COUNTER.load(Ordering::SeqCst) };
    // SAFETY: the buffer is the 0x98 bytes the wrapper allocated
    unsafe {
        buffer.cast::<usize>().write(STACK_VTABLE.load(Ordering::SeqCst));
        buffer.cast::<usize>().add(1).write(if known { counter } else { 0 });
        buffer.cast::<usize>().add(2).write(bundle.unwrap_or(0));
        buffer.cast::<u8>().add(0x22).write(5);
    }
    buffer
}

extern "C" fn dtor(this: *mut c_void, _flags: u32) -> *mut c_void {
    DTOR_CALLS.fetch_add(1, Ordering::SeqCst);
    this
}

// an item whose mFullName is `name`, and the counter that points to it
fn fake_item(name: &str) -> (Box<[u64; 16]>, Box<[usize; 1]>) {
    let mut item = Box::new([0u64; 16]);
    // SAFETY: the test heap is the Rust allocator; the string is never freed
    let text = unsafe { MsvcString::new_in(&TestHeap::heap(), name) }.unwrap();
    // SAFETY: the item block holds a HashedString at FULL_NAME
    unsafe {
        item.as_mut_ptr().cast::<u8>().add(FULL_NAME).cast::<HashedStringLayout>().write(HashedStringLayout::new(text))
    };
    let counter = Box::new([item.as_ptr() as usize]);
    (item, counter)
}

struct Setup {
    entries: GameEntries,
    _vtable: Box<[usize; 1]>,
    _items: [Box<[u64; 16]>; 2],
    _counters: [Box<[usize; 1]>; 2],
}

impl Setup {
    fn new() -> Self {
        let vtable = Box::new([dtor as *const () as usize]);
        let (stone, stone_counter) = fake_item("minecraft:stone");
        let (bundle, bundle_counter) = fake_item("minecraft:red_bundle");
        STACK_VTABLE.store(vtable.as_ptr() as usize, Ordering::SeqCst);
        ITEM_COUNTER.store(stone_counter.as_ptr() as usize, Ordering::SeqCst);
        BUNDLE_COUNTER.store(bundle_counter.as_ptr() as usize, Ordering::SeqCst);
        let entries = FakeResolver::entries(&[
            (EntryId::ITEM_STACK_FROM_TAG, from_tag as *const () as usize as u64),
            (EntryId::COMPOUND_TAG_NODE_VARIANT_INDEX, 0x68),
            (EntryId::ITEM_FULL_NAME, FULL_NAME as u64),
        ]);
        Self { entries, _vtable: vtable, _items: [stone, bundle], _counters: [stone_counter, bundle_counter] }
    }
}

fn entry(slot: Option<i8>) -> FakeCompound {
    let mut values = vec![("Count", FakeValue::Int(1))];
    if let Some(slot) = slot {
        values.push(("Slot", FakeValue::Byte(slot)));
    }
    FakeCompound::build(values)
}

fn items<'a>(tag: &FakeCompound, entries: &'a GameEntries) -> amml_game::NbtList<'a> {
    // SAFETY: a live fake laid out as the 26.60 CompoundTag
    let root = unsafe { CompoundTagRef::from_raw(tag.ptr(), entries) }.unwrap();
    root.list("Items").unwrap()
}

fn key() -> PreviewKey {
    PreviewKey { stack: 1, user_data: 2, count: 1 }
}

#[test]
fn a_box_with_bad_entries_shows_the_good_one_skips_three_and_logs_once() {
    LogCapture::install();
    let setup = Setup::new();
    let unknown = entry(Some(2));
    UNKNOWN_TAG.store(unknown.ptr() as usize, Ordering::SeqCst);
    let tag = FakeCompound::build(vec![(
        "Items",
        FakeValue::List(vec![entry(Some(0)), entry(None), entry(Some(40)), unknown]),
    )]);
    let source = GameBox::new(items(&tag, &setup.entries), TestHeap::heap());
    let mut cache = PreviewCache::new();

    cache.sync(key(), Some(&source));

    let shown = cache.item(0).unwrap();
    assert_eq!(shown.count_text.as_deref(), Some("5"));
    assert!(!shown.glint);
    assert!(shown.bar.is_none());
    assert!((1..27).all(|slot| cache.item(slot).is_none()));
    assert_eq!(cache.skipped(), 3);
    assert_eq!(LogCapture::count("shulker box entries skipped"), 1);

    let before = DTOR_CALLS.load(Ordering::SeqCst);
    cache.clear();

    assert_eq!(DTOR_CALLS.load(Ordering::SeqCst), before + 1);
}

#[test]
fn an_element_that_is_not_a_compound_is_skipped_and_the_next_one_still_shows() {
    let setup = Setup::new();
    UNKNOWN_TAG.store(0, Ordering::SeqCst);
    let tag = FakeCompound::build(vec![("Items", FakeValue::ListWithAByte(vec![entry(Some(9))]))]);
    let source = GameBox::new(items(&tag, &setup.entries), TestHeap::heap());
    let mut cache = PreviewCache::new();

    cache.sync(key(), Some(&source));

    assert!(cache.item(9).is_some());
    assert_eq!(cache.skipped(), 1);
}

// a bundle entry at `slot` whose stack gets `user_data` (the tag of the bundle), registered with the fake fromTag
fn bundle_entry(slot: i8, user_data: Option<&FakeCompound>) -> FakeCompound {
    let tag = entry(Some(slot));
    BUNDLES.lock().unwrap().push((tag.ptr() as usize, user_data.map_or(0, |data| data.ptr() as usize)));
    tag
}

fn contents(values: Vec<FakeCompound>) -> FakeCompound {
    FakeCompound::build(vec![
        ("bundle_weight", FakeValue::Int(40)),
        ("storage_item_component_content", FakeValue::List(values)),
    ])
}

fn box_of(entries: Vec<FakeCompound>) -> FakeCompound {
    FakeCompound::build(vec![("Items", FakeValue::List(entries))])
}

#[test]
fn a_bundle_is_marked_with_its_weight_and_its_contents_fill_the_strip_in_slot_order() {
    let setup = Setup::new();
    let user_data = contents(vec![entry(Some(2)), entry(Some(0)), entry(Some(1))]);
    let tag = box_of(vec![bundle_entry(3, Some(&user_data)), entry(Some(4))]);
    let source = GameBox::new(items(&tag, &setup.entries), TestHeap::heap());
    let mut cache = PreviewCache::new();

    cache.sync(key(), Some(&source));
    let before = DTOR_CALLS.load(Ordering::SeqCst);
    cache.sync_strip(Some(&source), Some(3));

    let bundle = cache.item(3).unwrap();
    assert!(bundle.bundle);
    assert_eq!(bundle.weight, Some(40));
    let plain = cache.item(4).unwrap();
    assert!(!plain.bundle);
    assert_eq!(plain.weight, None);
    assert_eq!(cache.bundles().len(), 1);
    assert_eq!(cache.bundles()[0].slot, 3);
    assert_eq!(cache.strip_len(), 3);
    assert!((0..3).all(|position| cache.strip_item(position).is_some()));
    // the stack made to read the contents is destroyed when the strip is built
    assert_eq!(DTOR_CALLS.load(Ordering::SeqCst), before + 1);

    cache.clear();

    assert_eq!(DTOR_CALLS.load(Ordering::SeqCst), before + 1 + 2 + 3);
}

#[test]
fn a_bundle_with_no_list_a_list_that_is_not_one_or_no_tag_gives_an_empty_strip_and_the_grid_stays() {
    let setup = Setup::new();
    let no_list = FakeCompound::build(vec![("bundle_weight", FakeValue::Int(1))]);
    let not_a_list = FakeCompound::build(vec![
        ("bundle_weight", FakeValue::Int(1)),
        ("storage_item_component_content", FakeValue::Int(5)),
    ]);
    let tag = box_of(vec![
        bundle_entry(0, Some(&no_list)),
        bundle_entry(1, Some(&not_a_list)),
        bundle_entry(2, None),
        entry(Some(3)),
    ]);
    let source = GameBox::new(items(&tag, &setup.entries), TestHeap::heap());
    let mut cache = PreviewCache::new();
    cache.sync(key(), Some(&source));

    for slot in 0..3 {
        cache.sync_strip(Some(&source), Some(slot));

        assert_eq!(cache.strip_len(), 0, "bundle in slot {slot}");
    }

    assert_eq!(cache.bundles().len(), 3);
    assert_eq!(cache.item(2).unwrap().weight, None);
    assert!(cache.item(3).is_some());
    assert_eq!(cache.skipped(), 0);
}

#[test]
fn bad_elements_of_a_contents_list_lose_only_themselves() {
    LogCapture::install();
    let setup = Setup::new();
    let user_data = FakeCompound::build(vec![(
        "storage_item_component_content",
        FakeValue::ListWithAByte(vec![entry(Some(1)), entry(None), entry(Some(0))]),
    )]);
    let tag = box_of(vec![bundle_entry(0, Some(&user_data))]);
    let source = GameBox::new(items(&tag, &setup.entries), TestHeap::heap());
    let mut cache = PreviewCache::new();
    cache.sync(key(), Some(&source));

    cache.sync_strip(Some(&source), Some(0));

    assert_eq!(cache.strip_len(), 3);
    let shown: Vec<bool> = (0..4).map(|position| cache.strip_item(position).is_some()).collect();
    assert_eq!(shown, [true, true, true, false]);
    assert_eq!(LogCapture::count("bundle entries skipped"), 0);
    assert!(cache.item(0).is_some());
}

#[test]
fn a_selected_slot_that_holds_no_bundle_gives_an_empty_strip() {
    let setup = Setup::new();
    let tag = box_of(vec![entry(Some(4))]);
    let source = GameBox::new(items(&tag, &setup.entries), TestHeap::heap());
    let mut cache = PreviewCache::new();
    cache.sync(key(), Some(&source));

    cache.sync_strip(Some(&source), Some(4));
    cache.sync_strip(Some(&source), Some(9));

    assert_eq!(cache.strip_len(), 0);
}
