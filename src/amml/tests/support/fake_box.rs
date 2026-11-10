use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use alaydriem_peek::traits::BoxSource;
use alaydriem_peek::types::PreviewItem;
use peek_core::structs::DurabilityBar;

use super::TrackedStack;

// one entry of a fake shulker box: the Slot it carries and the item it builds, if the game knows the name
#[derive(Clone)]
pub struct FakeEntry {
    pub slot: Option<i32>,
    pub item: Option<&'static str>,
    pub glint: bool,
    pub bar: Option<DurabilityBar>,
    pub count_text: Option<&'static str>,
    pub bundle: bool,
    pub weight: Option<i32>,
    // the entries inside a bundle; `None` is a bundle whose list is missing
    pub contents: Option<Vec<FakeEntry>>,
}

impl FakeEntry {
    pub fn item(slot: Option<i32>, name: &'static str) -> Self {
        Self {
            slot,
            item: Some(name),
            glint: false,
            bar: None,
            count_text: None,
            bundle: false,
            weight: None,
            contents: None,
        }
    }

    // an entry whose item name the game does not know: building gives nothing
    pub fn unknown(slot: Option<i32>) -> Self {
        Self {
            slot,
            item: None,
            glint: false,
            bar: None,
            count_text: None,
            bundle: false,
            weight: None,
            contents: None,
        }
    }

    // a bundle with this weight and these entries
    pub fn bundle(mut self, weight: Option<i32>, contents: Option<Vec<FakeEntry>>) -> Self {
        self.bundle = true;
        self.weight = weight;
        self.contents = contents;
        self
    }

    pub fn glinting(mut self) -> Self {
        self.glint = true;
        self
    }

    pub fn damaged(mut self, damage: i32, max_damage: i32) -> Self {
        self.bar = DurabilityBar::new(true, damage, max_damage);
        self
    }

    pub fn counted(mut self, text: &'static str) -> Self {
        self.count_text = Some(text);
        self
    }
}

// a shulker box whose entries are plain data; it records which entries were built
pub struct FakeBox {
    entries: Vec<FakeEntry>,
    drops: Rc<Cell<usize>>,
    built: RefCell<Vec<usize>>,
}

impl FakeBox {
    pub fn new(drops: &Rc<Cell<usize>>, entries: Vec<FakeEntry>) -> Self {
        Self { entries, drops: Rc::clone(drops), built: RefCell::new(Vec::new()) }
    }

    pub fn built(&self) -> Vec<usize> {
        self.built.borrow().clone()
    }
}

impl BoxSource for FakeBox {
    type Stack = TrackedStack;
    type Contents = FakeBox;

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn slot(&self, index: usize) -> Option<i32> {
        self.entries.get(index).and_then(|entry| entry.slot)
    }

    fn build(&self, index: usize) -> Option<PreviewItem<TrackedStack>> {
        self.built.borrow_mut().push(index);
        let entry = self.entries.get(index)?;
        let name = entry.item?;
        Some(PreviewItem {
            stack: TrackedStack::new(name, &self.drops),
            glint: entry.glint,
            bar: entry.bar,
            count_text: entry.count_text.map(str::to_string),
            bundle: entry.bundle,
            weight: entry.weight,
        })
    }

    fn bundle_contents(&self, index: usize) -> Option<FakeBox> {
        let entry = self.entries.get(index)?;
        let contents = entry.contents.clone()?;
        entry.bundle.then(|| FakeBox::new(&self.drops, contents))
    }
}
