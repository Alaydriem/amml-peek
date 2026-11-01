use amml_game::ContainerScreenController;
use amml_msvc::{MsvcHeap, MsvcString};
use peek_core::services::ShulkerName;
use peek_core::structs::HoveredSlot;

use crate::game::GameBox;
use crate::traits::BoxResolver;
use crate::types::{PreviewKey, ResolvedBox};

/// Finds the shulker box in the slot the pointer is over, through the container screen controller. Client main
/// thread; the stack belongs to the game and is read only.
pub struct GameBoxResolver<'a> {
    container: &'a ContainerScreenController<'static>,
    heap: MsvcHeap,
}

impl<'a> GameBoxResolver<'a> {
    /// `heap` must be the game heap.
    pub fn new(container: &'a ContainerScreenController<'static>, heap: MsvcHeap) -> Self {
        Self { container, heap }
    }
}

impl BoxResolver for GameBoxResolver<'_> {
    type Source = GameBox<'static>;

    #[allow(unsafe_code)]
    fn resolve(&self, slot: &HoveredSlot) -> Option<ResolvedBox<GameBox<'static>>> {
        // SAFETY: the heap is the game's operator new and delete.
        let mut name = unsafe { MsvcString::new_in(&self.heap, &slot.collection) }.ok()?;
        let stack = self.container.visual_item(&name, slot.index);
        // SAFETY: the string came from this heap, and the call that read it has returned.
        unsafe { name.release(&self.heap) };
        let stack = stack?;
        if stack.is_empty() || !ShulkerName::is_shulker(&stack.type_name()?) {
            return None;
        }
        let user_data = stack.user_data();
        let key = PreviewKey {
            stack: stack.as_ptr() as usize,
            user_data: user_data.as_ref().map_or(0, |tag| tag.as_ptr() as usize),
            count: stack.count()?,
        };
        let contents = user_data.and_then(|tag| tag.list("Items")).map(|list| GameBox::new(list, self.heap));
        Some(ResolvedBox { key, contents })
    }
}
