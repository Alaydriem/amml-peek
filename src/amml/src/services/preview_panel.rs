use std::fmt::Write;

use peek_core::services::{BundleSelection, HoverTracker, StripLayout};
use peek_core::structs::{PreviewLayout, SlotCollection};

use crate::services::{PreviewCache, PreviewRenderer, ScrollGate};
use crate::traits::{BoxResolver, BoxSource, Canvas};
use crate::types::{BoxSummary, DrawOutcome, ManagerDelta, ViewFrame};

/// The hover and the items of the box the pointer is over: decides whether a render call draws the panel, and draws it.
/// The hint text is the one buffer the panel owns; it is written when the selected bundle changes. The box is read
/// from the game once for each key and kept as `source`, which the strip of any bundle is built from.
pub struct PreviewPanel<B: BoxSource> {
    tracker: HoverTracker,
    cache: PreviewCache<B::Stack>,
    source: Option<B>,
    // whether the stack the cache key names is a shulker box
    is_box: bool,
    // set when a box was loaded, taken by `take_loaded`
    loaded: Option<BoxSummary>,
    selection: BundleSelection,
    // whether the wheel hook is installed; without it the hint is not drawn
    scroll: bool,
    hint: String,
    // the position the text in `hint` was written for
    hint_for: Option<(usize, usize)>,
}

impl<B: BoxSource> PreviewPanel<B> {
    const COLUMNS: u8 = 9;
    const ROWS: u8 = 3;
    // the longest hint: the text and two 2-digit numbers
    const HINT_CAPACITY: usize = 48;

    pub fn new() -> Self {
        Self {
            tracker: HoverTracker::new(),
            cache: PreviewCache::new(),
            source: None,
            is_box: false,
            loaded: None,
            selection: BundleSelection::new(),
            scroll: false,
            hint: String::with_capacity(Self::HINT_CAPACITY),
            hint_for: None,
        }
    }

    /// Whether the wheel hook is installed.
    pub fn set_scroll(&mut self, on: bool) {
        self.scroll = on;
    }

    /// A slot other than the current one ends the selection at once, so no wheel notch before the next draw is
    /// taken for the box that was there.
    pub fn hovered(&mut self, screen: u64, collection: SlotCollection, slot: i32) {
        let same = self
            .tracker
            .current_for(screen)
            .is_some_and(|current| current.collection == collection && current.index == slot);
        if !same {
            self.selection.reset(&[]);
        }
        self.tracker.hovered(screen, collection, slot);
    }

    pub fn unhovered(&mut self, screen: u64, collection: SlotCollection, slot: i32) {
        self.tracker.unhovered(screen, collection, slot);
    }

    /// The screen `screen` closed. When it is the hovered screen, the items are dropped and the selection ends;
    /// another screen closing changes nothing.
    pub fn left(&mut self, screen: u64) {
        if self.tracker.screen() != Some(screen) {
            return;
        }
        self.tracker.left(screen);
        self.drop_box();
        self.selection.reset(&[]);
    }

    /// Drops the items and keeps the hover: the next draw reads the box again.
    pub fn reload(&mut self) {
        self.drop_box();
        self.selection.reset(&[]);
    }

    /// Forgets the hover and drops the items.
    pub fn clear(&mut self) {
        self.tracker.clear();
        self.drop_box();
        self.selection.reset(&[]);
    }

    /// Applies the manager's events: a disable forgets the hover and drops the items, on the calling thread.
    pub fn apply(&mut self, delta: ManagerDelta) {
        if delta.disabled {
            self.clear();
        }
    }

    pub fn is_tracking(&self, screen: u64) -> bool {
        self.tracker.current_for(screen).is_some()
    }

    /// The box loaded since the last call, once.
    pub fn take_loaded(&mut self) -> Option<BoxSummary> {
        self.loaded.take()
    }

    /// The 1-based index of the selected bundle and the number of bundles.
    pub fn selected_position(&self) -> Option<(usize, usize)> {
        self.selection.position()
    }

    /// One wheel notch on the scene of `scene_screen`. True when peek takes it: the pointer is over a box with
    /// two or more bundles on that same screen and Shift is held. The selection then moves and the strip of the old
    /// bundle is dropped.
    pub fn scroll(&mut self, scene_screen: Option<u64>, shift: bool, notches: i32) -> bool {
        let consume = self.scroll
            && ScrollGate::consume(self.tracker.screen(), scene_screen, self.selection.count(), shift, notches);
        if consume && self.selection.step(notches) {
            self.cache.drop_strip();
        }
        consume
    }

    /// Draws the panel when the pointer is over a slot of `frame.screen` that holds a shulker box. Nothing is
    /// resolved or drawn for any other screen. The box is read from the game only when the key of the stack changes.
    pub fn draw<R, C>(&mut self, frame: &ViewFrame, resolver: &R, canvas: &mut C) -> DrawOutcome
    where
        R: BoxResolver<Source = B>,
        C: Canvas<Stack = B::Stack>,
    {
        let Some(slot) = self.tracker.current_for(frame.screen).copied() else {
            self.selection.reset(&[]);
            // no slot is hovered on any screen: the box is read again at the next hover, so a stack that was swapped
            // meanwhile is not shown from the old copy
            if self.tracker.screen().is_none() {
                self.drop_box();
            }
            return DrawOutcome::Idle;
        };
        let Some(key) = resolver.key(&slot) else {
            self.drop_box();
            self.selection.reset(&[]);
            return DrawOutcome::Skipped;
        };
        let changed = self.cache.key() != Some(key);
        if changed {
            // the old copies go before the new ones are made: the game gives out a limited number of them
            self.drop_box();
            let resolved = resolver.load(&slot);
            self.is_box = resolved.is_some();
            self.source = resolved.and_then(|resolved| resolved.contents);
            self.cache.sync(key, self.source.as_ref());
            if self.is_box {
                let items = (0..PreviewCache::<B::Stack>::SLOTS).filter(|slot| self.cache.item(*slot).is_some());
                self.loaded = Some(BoxSummary { items: items.count(), bundles: self.cache.bundles().len() });
            }
        }
        if !self.is_box {
            self.selection.reset(&[]);
            return DrawOutcome::Skipped;
        }
        if changed || self.selection.selected().is_none() {
            self.select_first();
        }
        let selected = self.selection.selected();
        self.cache.sync_strip(self.source.as_ref(), selected);
        let position = self.selection.position().filter(|(_, count)| self.scroll && *count >= 2);
        self.write_hint(position);
        let strip_rows = if selected.is_some() { StripLayout::rows(self.cache.strip_len()) } else { 0 };
        let layout = PreviewLayout::anchored_panel(
            Self::COLUMNS,
            Self::ROWS,
            strip_rows,
            position.is_some(),
            frame.pointer,
            frame.size,
        );
        let hint = position.map(|_| self.hint.as_str());
        PreviewRenderer::draw(&layout, &self.cache, selected, hint, canvas);
        DrawOutcome::Drawn
    }

    // drops the box, its items and the key, so the next draw loads the box again
    fn drop_box(&mut self) {
        self.cache.clear();
        self.source = None;
        self.is_box = false;
    }

    // selects the first bundle of the box in the cache
    fn select_first(&mut self) {
        let mut slots = [0u8; BundleSelection::CAPACITY];
        let bundles = self.cache.bundles();
        for (target, bundle) in slots.iter_mut().zip(bundles) {
            *target = bundle.slot;
        }
        self.selection.reset(&slots[..bundles.len().min(slots.len())]);
    }

    // writes the hint for `position` unless it is already there
    fn write_hint(&mut self, position: Option<(usize, usize)>) {
        if position == self.hint_for {
            return;
        }
        self.hint_for = position;
        self.hint.clear();
        if let Some((index, count)) = position {
            let _ = write!(self.hint, "Shift + scroll: next bundle ({index}/{count})");
        }
    }
}

impl<B: BoxSource> Default for PreviewPanel<B> {
    fn default() -> Self {
        Self::new()
    }
}
