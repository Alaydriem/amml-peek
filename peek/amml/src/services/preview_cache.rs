use peek_core::services::StripLayout;
use peek_core::structs::{BundleEntry, ShulkerGrid};

use crate::traits::BoxSource;
use crate::types::{PreviewItem, PreviewKey};

/// The items of the box the pointer is over, and of the selected bundle in it. Bound: one box, `SLOTS` items, and
/// one bundle, `STRIP_SLOTS` items. Eviction: the box items and the bundle items are dropped when the key changes,
/// on `clear` and with the cache; the bundle items are also dropped when the selected bundle changes.
pub struct PreviewCache<S> {
    key: Option<PreviewKey>,
    grid: ShulkerGrid,
    slots: [Option<PreviewItem<S>>; ShulkerGrid::COLUMNS * ShulkerGrid::ROWS],
    skipped: usize,
    bundles: [BundleEntry; ShulkerGrid::COLUMNS * ShulkerGrid::ROWS],
    bundle_count: usize,
    // the box slot of the bundle the strip was built for; `None` before the first build and after a drop
    strip_for: Option<u8>,
    strip: [Option<PreviewItem<S>>; StripLayout::MAX_ENTRIES],
    strip_len: usize,
}

impl<S> PreviewCache<S> {
    pub const SLOTS: usize = ShulkerGrid::COLUMNS * ShulkerGrid::ROWS;
    pub const STRIP_SLOTS: usize = StripLayout::MAX_ENTRIES;

    pub fn new() -> Self {
        Self {
            key: None,
            grid: ShulkerGrid::new(),
            slots: std::array::from_fn(|_| None),
            skipped: 0,
            bundles: [BundleEntry { slot: 0, weight: None }; ShulkerGrid::COLUMNS * ShulkerGrid::ROWS],
            bundle_count: 0,
            strip_for: None,
            strip: std::array::from_fn(|_| None),
            strip_len: 0,
        }
    }

    /// Makes the cache hold the contents of the box `key` names; true when the box changed. The same key as before
    /// changes nothing. A new key drops the old items, places each entry by its `Slot` and builds the placed items;
    /// an entry with no usable slot, or one the game cannot make an item of, is skipped, and the skips of the box
    /// are logged once. The bundles among the built items are noted in slot order.
    pub fn sync<B: BoxSource<Stack = S>>(&mut self, key: PreviewKey, contents: Option<&B>) -> bool {
        if self.key == Some(key) {
            return false;
        }
        self.clear();
        self.key = Some(key);
        let Some(contents) = contents else { return true };
        let mut skipped = 0;
        for index in 0..contents.len() {
            let placed = contents.slot(index).is_some_and(|slot| self.grid.place(slot, index));
            if !placed {
                skipped += 1;
            }
        }
        for (slot, target) in self.slots.iter_mut().enumerate() {
            let Some(entry) = self.grid.entry_at(slot) else { continue };
            match contents.build(entry) {
                Some(item) => {
                    if item.bundle {
                        self.bundles[self.bundle_count] = BundleEntry { slot: slot as u8, weight: item.weight };
                        self.bundle_count += 1;
                    }
                    *target = Some(item);
                }
                None => skipped += 1,
            }
        }
        self.skipped = skipped;
        if skipped > 0 {
            curia::warn!("shulker box entries skipped", { skipped: skipped as u64 });
        }
        true
    }

    /// Makes the strip hold the contents of the bundle in box slot `selected`. The same slot as before changes
    /// nothing. Otherwise the old strip is dropped at once and the new one built: the entries of the bundle in
    /// `Slot` order, the first `STRIP_SLOTS` of them, packed with no gaps. An entry the game cannot make an item of
    /// is left out: a bundle's list holds an empty entry for each unused place. A bundle with no usable list gives
    /// an empty strip.
    pub fn sync_strip<B: BoxSource<Stack = S>>(&mut self, contents: Option<&B>, selected: Option<u8>) {
        if self.strip_for == selected {
            return;
        }
        self.drop_strip();
        self.strip_for = selected;
        let Some(selected) = selected else { return };
        let Some(source) = contents
            .zip(self.grid.entry_at(usize::from(selected)))
            .and_then(|(contents, entry)| contents.bundle_contents(entry))
        else {
            return;
        };
        let mut order = [0usize; StripLayout::MAX_ENTRIES];
        let count = StripLayout::order(source.len(), |index| source.slot(index), &mut order);
        for index in &order[..count] {
            if let Some(item) = source.build(*index) {
                self.strip[self.strip_len] = Some(item);
                self.strip_len += 1;
            }
        }
    }

    pub fn item(&self, slot: usize) -> Option<&PreviewItem<S>> {
        self.slots.get(slot).and_then(Option::as_ref)
    }

    /// The item in strip cell `position`.
    pub fn strip_item(&self, position: usize) -> Option<&PreviewItem<S>> {
        self.strip.get(position).and_then(Option::as_ref)
    }

    /// The number of items in the strip.
    pub fn strip_len(&self) -> usize {
        self.strip_len
    }

    /// The bundles of the current box, in slot order.
    pub fn bundles(&self) -> &[BundleEntry] {
        &self.bundles[..self.bundle_count]
    }

    /// The entries skipped while building the current box.
    pub fn skipped(&self) -> usize {
        self.skipped
    }

    pub fn key(&self) -> Option<PreviewKey> {
        self.key
    }

    /// Drops the strip items now; the next `sync_strip` builds the strip again.
    pub fn drop_strip(&mut self) {
        self.strip_for = None;
        self.strip_len = 0;
        for cell in &mut self.strip {
            *cell = None;
        }
    }

    pub fn clear(&mut self) {
        self.key = None;
        self.skipped = 0;
        self.grid = ShulkerGrid::new();
        self.bundle_count = 0;
        for slot in &mut self.slots {
            *slot = None;
        }
        self.drop_strip();
    }
}

impl<S> Default for PreviewCache<S> {
    fn default() -> Self {
        Self::new()
    }
}
