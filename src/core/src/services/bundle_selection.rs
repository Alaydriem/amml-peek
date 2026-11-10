/// Which bundle of the hovered shulker box the strip shows. Bound: the 27 slots of one box; `reset` replaces them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleSelection {
    slots: [u8; Self::CAPACITY],
    count: usize,
    index: usize,
}

impl BundleSelection {
    pub const CAPACITY: usize = 27;

    pub fn new() -> BundleSelection {
        BundleSelection { slots: [0; Self::CAPACITY], count: 0, index: 0 }
    }

    /// Takes the box slots of the bundles, in slot order, and selects the first. Slots past `CAPACITY` are ignored.
    pub fn reset(&mut self, bundles: &[u8]) {
        let count = bundles.len().min(Self::CAPACITY);
        self.slots[..count].copy_from_slice(&bundles[..count]);
        self.count = count;
        self.index = 0;
    }

    /// The box slot of the selected bundle.
    pub fn selected(&self) -> Option<u8> {
        (self.index < self.count).then(|| self.slots[self.index])
    }

    /// Moves the selection by `notches`, positive to the next bundle, wrapping at both ends. True when the
    /// selection changed; with fewer than two bundles it never does.
    pub fn step(&mut self, notches: i32) -> bool {
        if self.count < 2 {
            return false;
        }
        let count = self.count as i64;
        let next = (self.index as i64 + i64::from(notches)).rem_euclid(count) as usize;
        let changed = next != self.index;
        self.index = next;
        changed
    }

    /// The 1-based index of the selected bundle and the number of bundles.
    pub fn position(&self) -> Option<(usize, usize)> {
        (self.count > 0).then_some((self.index + 1, self.count))
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl Default for BundleSelection {
    fn default() -> Self {
        Self::new()
    }
}
