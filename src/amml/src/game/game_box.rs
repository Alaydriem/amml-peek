use std::sync::Arc;

use amml_api::{ContainerContents, ContainerEntry, ItemDrawHandle};
use peek_core::services::{BundleName, CountLabel};
use peek_core::structs::DurabilityBar;

use crate::game::BoxEntry;
use crate::traits::BoxSource;
use crate::types::PreviewItem;

/// The entries of one shulker box or one bundle, from the contents the loader copied. The draw handles inside live as
/// long as the box does: dropping the box gives them back to the loader. Game thread.
pub struct GameBox {
    entries: Arc<[BoxEntry]>,
}

impl GameBox {
    pub fn new(contents: ContainerContents) -> Self {
        Self { entries: Self::entries_of(contents) }
    }

    fn entries_of(contents: ContainerContents) -> Arc<[BoxEntry]> {
        contents.entries.into_iter().map(Self::entry_of).collect()
    }

    fn entry_of(entry: ContainerEntry) -> BoxEntry {
        let ContainerEntry { slot, item, draw, bundle, bundle_weight } = entry;
        let bundle_item = item.type_name.as_deref().is_some_and(BundleName::is_bundle);
        BoxEntry {
            slot,
            count: item.count,
            glint: item.is_glint.unwrap_or(false),
            bar: DurabilityBar::new(
                item.damage.is_some_and(|damage| damage > 0),
                item.damage.unwrap_or(0),
                item.max_damage.unwrap_or(0),
            ),
            bundle: bundle_item,
            weight: if bundle_item { bundle_weight } else { None },
            draw: Arc::new(draw),
            contents: bundle.map(Self::entries_of),
        }
    }
}

impl BoxSource for GameBox {
    type Stack = Arc<ItemDrawHandle>;
    type Contents = GameBox;

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn slot(&self, index: usize) -> Option<i32> {
        self.entries.get(index).map(|entry| i32::from(entry.slot))
    }

    fn build(&self, index: usize) -> Option<PreviewItem<Arc<ItemDrawHandle>>> {
        let entry = self.entries.get(index)?;
        Some(PreviewItem {
            stack: Arc::clone(&entry.draw),
            glint: entry.glint,
            bar: entry.bar,
            count_text: CountLabel::text(entry.count),
            bundle: entry.bundle,
            weight: entry.weight,
        })
    }

    fn bundle_contents(&self, index: usize) -> Option<GameBox> {
        let entry = self.entries.get(index)?;
        let entries = Arc::clone(entry.contents.as_ref()?);
        entry.bundle.then_some(GameBox { entries })
    }
}
