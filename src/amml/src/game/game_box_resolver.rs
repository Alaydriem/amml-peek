use amml_api::{Collection, ContainerScreen, ScreenId, SlotAt};
use peek_core::services::ShulkerName;
use peek_core::structs::{HoveredSlot, SlotCollection};

use crate::game::GameBox;
use crate::traits::BoxResolver;
use crate::types::{PreviewKey, ResolvedBox};

/// Finds the shulker box in the slot the pointer is over, through the container screen of the running render call.
/// Game thread; the stack belongs to the game and is read only.
pub struct GameBoxResolver<'a, 'f> {
    screen: &'a ContainerScreen<'f>,
}

impl<'a, 'f> GameBoxResolver<'a, 'f> {
    pub fn new(screen: &'a ContainerScreen<'f>) -> Self {
        Self { screen }
    }

    /// The slot the API names for a hovered slot; `None` for a collection the API does not serve.
    pub fn slot_at(slot: &HoveredSlot) -> Option<SlotAt> {
        let collection = match slot.collection {
            SlotCollection::Container => Collection::Container,
            SlotCollection::Hotbar => Collection::Hotbar,
            SlotCollection::Inventory => Collection::Inventory,
            SlotCollection::Other => return None,
        };
        Some(SlotAt { collection, index: slot.index })
    }
}

impl BoxResolver for GameBoxResolver<'_, '_> {
    type Source = GameBox;

    fn key(&self, slot: &HoveredSlot) -> Option<PreviewKey> {
        let at = Self::slot_at(slot)?;
        let stack = self.screen.visual_item(at)?;
        if stack.is_empty() {
            return None;
        }
        Some(PreviewKey {
            screen: ScreenId(slot.screen),
            slot: at,
            identity: stack.identity(),
            aux: stack.aux(),
            count: stack.count(),
            change_stamp: stack.change_stamp(),
        })
    }

    fn load(&self, slot: &HoveredSlot) -> Option<ResolvedBox<GameBox>> {
        let stack = self.screen.visual_item(Self::slot_at(slot)?)?;
        if stack.is_empty() || !ShulkerName::is_shulker(&stack.type_name()?) {
            return None;
        }
        Some(ResolvedBox { contents: stack.container_contents().map(GameBox::new) })
    }
}
