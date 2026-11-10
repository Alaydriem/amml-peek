/// The grid of the selected bundle's contents under the box grid.
pub struct StripLayout;

impl StripLayout {
    pub const COLUMNS: usize = 9;
    /// The most a bundle holds.
    pub const MAX_ENTRIES: usize = 64;

    /// Rows for `entries` entries; an empty strip is one row, and entries past `MAX_ENTRIES` add none.
    pub fn rows(entries: usize) -> usize {
        entries.min(Self::MAX_ENTRIES).div_ceil(Self::COLUMNS).max(1)
    }

    /// Column and row of the cell at `index`; `None` from `MAX_ENTRIES` on.
    pub fn cell(index: usize) -> Option<(usize, usize)> {
        (index < Self::MAX_ENTRIES).then_some((index % Self::COLUMNS, index / Self::COLUMNS))
    }

    /// Writes the list indexes of the first `MAX_ENTRIES` of `len` entries into `out` in `Slot` order. Entries with
    /// no slot or a negative one come after the others, in list order, as do equal slots. Returns how many were
    /// written.
    pub fn order(len: usize, slot: impl Fn(usize) -> Option<i32>, out: &mut [usize; Self::MAX_ENTRIES]) -> usize {
        let count = len.min(Self::MAX_ENTRIES);
        let mut keys = [(true, 0i32, 0usize); Self::MAX_ENTRIES];
        for (index, key) in keys.iter_mut().enumerate().take(count) {
            let slot = slot(index).filter(|slot| *slot >= 0);
            *key = (slot.is_none(), slot.unwrap_or(0), index);
        }
        keys[..count].sort_unstable();
        for (target, key) in out.iter_mut().zip(&keys[..count]) {
            *target = key.2;
        }
        count
    }
}
