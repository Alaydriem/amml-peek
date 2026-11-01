/// The 27 slots of one shulker box. Each slot holds an index into the caller's entry list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShulkerGrid {
    slots: [Option<usize>; 27],
}

impl ShulkerGrid {
    pub const COLUMNS: usize = 9;
    pub const ROWS: usize = 3;

    pub fn new() -> ShulkerGrid {
        ShulkerGrid { slots: [None; Self::COLUMNS * Self::ROWS] }
    }

    /// False and no change when `slot` is outside the box. A later entry replaces an earlier one.
    pub fn place(&mut self, slot: i32, entry: usize) -> bool {
        match usize::try_from(slot).ok().and_then(|slot| self.slots.get_mut(slot)) {
            Some(target) => {
                *target = Some(entry);
                true
            }
            None => false,
        }
    }

    pub fn entry_at(&self, slot: usize) -> Option<usize> {
        self.slots.get(slot).copied().flatten()
    }

    pub fn filled(&self) -> usize {
        self.slots.iter().flatten().count()
    }
}

impl Default for ShulkerGrid {
    fn default() -> Self {
        Self::new()
    }
}
