use std::fmt::Write;

use peek_core::services::{BundleSelection, HoverTracker, StripLayout};
use peek_core::structs::PreviewLayout;

use crate::services::{PreviewCache, PreviewRenderer, ScrollGate};
use crate::traits::{BoxResolver, BoxSource, Canvas};
use crate::types::{DrawOutcome, ManagerDelta, ViewFrame};

/// The hover and the items of the box under the pointer: decides whether a render call draws the panel, and draws it.
/// The hint text is the one buffer the panel owns; it is written when the selected bundle changes.
pub struct PreviewPanel<S> {
    tracker: HoverTracker,
    cache: PreviewCache<S>,
    selection: BundleSelection,
    // whether the wheel hook is installed; without it the hint is not drawn
    scroll: bool,
    hint: String,
    // the position the text in `hint` was written for
    hint_for: Option<(usize, usize)>,
}

impl<S> PreviewPanel<S> {
    const COLUMNS: u8 = 9;
    const ROWS: u8 = 3;
    // the longest hint: the text and two 2-digit numbers
    const HINT_CAPACITY: usize = 48;

    pub fn new() -> Self {
        Self {
            tracker: HoverTracker::new(),
            cache: PreviewCache::new(),
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
    pub fn hovered(&mut self, controller: usize, collection: &str, slot: i32) {
        let same = self
            .tracker
            .current_for(controller)
            .is_some_and(|current| current.collection == collection && current.index == slot);
        if !same {
            self.selection.reset(&[]);
        }
        self.tracker.hovered(controller, collection, slot);
    }

    pub fn unhovered(&mut self, controller: usize, collection: &str, slot: i32) {
        self.tracker.unhovered(controller, collection, slot);
    }

    /// The screen of `controller` closed: no read goes through it again. When it is the hovered screen, the items
    /// are dropped and the selection ends; another screen closing changes nothing.
    pub fn left(&mut self, controller: usize) {
        if self.tracker.controller() != Some(controller) {
            return;
        }
        self.tracker.left(controller);
        self.cache.clear();
        self.selection.reset(&[]);
    }

    /// Forgets the hover and drops the items.
    pub fn clear(&mut self) {
        self.tracker.clear();
        self.cache.clear();
        self.selection.reset(&[]);
    }

    /// Applies the manager's events: a disable forgets the hover and drops the items, on the calling thread.
    pub fn apply(&mut self, delta: ManagerDelta) {
        if delta.disabled {
            self.clear();
        }
    }

    pub fn is_tracking(&self, controller: usize) -> bool {
        self.tracker.current_for(controller).is_some()
    }

    /// One wheel notch on the scene of `scene_controller`. True when peek takes it: the pointer is over a box with
    /// two or more bundles on that same screen and Shift is held. The selection then moves and the strip of the old
    /// bundle is dropped.
    pub fn scroll(&mut self, scene_controller: Option<usize>, shift: bool, notches: i32) -> bool {
        let consume = self.scroll
            && ScrollGate::consume(self.tracker.controller(), scene_controller, self.selection.count(), shift, notches);
        if consume && self.selection.step(notches) {
            self.cache.drop_strip();
        }
        consume
    }

    /// Draws the panel when the pointer is over a slot of `frame.controller` that holds a shulker box. Nothing is
    /// resolved or drawn for any other screen.
    pub fn draw<R, C>(&mut self, frame: &ViewFrame, resolver: &R, canvas: &mut C) -> DrawOutcome
    where
        R: BoxResolver,
        R::Source: BoxSource<Stack = S>,
        C: Canvas<Stack = S>,
    {
        let Some(slot) = self.tracker.current_for(frame.controller) else {
            self.selection.reset(&[]);
            return DrawOutcome::Idle;
        };
        let Some(resolved) = resolver.resolve(slot) else {
            self.cache.clear();
            self.selection.reset(&[]);
            return DrawOutcome::Skipped;
        };
        let changed = self.cache.sync(resolved.key, resolved.contents.as_ref());
        if changed || self.selection.selected().is_none() {
            self.select_first();
        }
        let selected = self.selection.selected();
        self.cache.sync_strip(resolved.contents.as_ref(), selected);
        let position = self.selection.position().filter(|(_, count)| self.scroll && *count >= 2);
        self.write_hint(position);
        let strip_rows = if selected.is_some() { StripLayout::rows(self.cache.strip_len()) } else { 0 };
        let layout = PreviewLayout::anchored_panel(
            Self::COLUMNS,
            Self::ROWS,
            strip_rows,
            position.is_some(),
            frame.pointer,
            frame.screen,
        );
        let hint = position.map(|_| self.hint.as_str());
        PreviewRenderer::draw(&layout, &self.cache, selected, hint, canvas);
        DrawOutcome::Drawn
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

impl<S> Default for PreviewPanel<S> {
    fn default() -> Self {
        Self::new()
    }
}
