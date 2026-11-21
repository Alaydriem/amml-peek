use std::cell::{Cell, RefCell};
use std::rc::Rc;

use alaydriem_peek::traits::BoxResolver;
use alaydriem_peek::types::{PreviewKey, ResolvedBox};
use amml_api::{Collection, ScreenId, SlotAt};
use peek_core::structs::HoveredSlot;

use super::{FakeBox, FakeEntry};

// resolves every slot to one stack: `None` contents is a stack that is no shulker box, an empty list a box with no
// items. The key changes only when the test says so (`next_box`, `set`), as the identity of a real stack does. It
// records the slots `load` was asked about and how many times `key` was read
pub struct FakeResolver {
    drops: Rc<Cell<usize>>,
    contents: RefCell<Option<Vec<FakeEntry>>>,
    key: Cell<u64>,
    keys: Cell<usize>,
    loaded: RefCell<Vec<HoveredSlot>>,
}

impl FakeResolver {
    pub fn new(drops: &Rc<Cell<usize>>, contents: Option<Vec<FakeEntry>>) -> Self {
        Self {
            drops: Rc::clone(drops),
            contents: RefCell::new(contents),
            key: Cell::new(0x1000),
            keys: Cell::new(0),
            loaded: RefCell::new(Vec::new()),
        }
    }

    pub fn loaded(&self) -> Vec<HoveredSlot> {
        self.loaded.borrow().clone()
    }

    pub fn keys(&self) -> usize {
        self.keys.get()
    }

    // a new stack in the slot: the key changes
    pub fn set(&self, contents: Option<Vec<FakeEntry>>) {
        *self.contents.borrow_mut() = contents;
        self.next_box();
    }

    pub fn next_box(&self) {
        self.key.set(self.key.get() + 1);
    }
}

impl BoxResolver for FakeResolver {
    type Source = FakeBox;

    fn key(&self, slot: &HoveredSlot) -> Option<PreviewKey> {
        self.keys.set(self.keys.get() + 1);
        Some(PreviewKey {
            screen: ScreenId(slot.screen),
            slot: SlotAt { collection: Collection::Container, index: slot.index },
            identity: Some(self.key.get()),
            aux: None,
            count: 1,
            change_stamp: None,
        })
    }

    fn load(&self, slot: &HoveredSlot) -> Option<ResolvedBox<FakeBox>> {
        self.loaded.borrow_mut().push(*slot);
        let entries = self.contents.borrow().clone()?;
        let contents = (!entries.is_empty()).then(|| FakeBox::new(&self.drops, entries));
        Some(ResolvedBox { contents })
    }
}
