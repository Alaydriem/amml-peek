use amml_game::{NbtList, OwnedItemStack};
use amml_msvc::MsvcHeap;
use peek_core::services::{BundleName, CountLabel};
use peek_core::structs::DurabilityBar;

use crate::traits::BoxSource;
use crate::types::PreviewItem;

/// A list of entries in the game, each an item tag with a `Slot` byte: the `Items` list of a shulker box, or the
/// contents list of a bundle. Items are made with `ItemStack::fromTag` into blocks of `heap`, which must be the game
/// heap. Client main thread.
pub struct GameBox<'a> {
    list: NbtList<'a>,
    heap: MsvcHeap,
    // the bundle stack whose user data holds `list`, kept alive for as long as the list is read
    _owner: Option<OwnedItemStack<'a>>,
}

impl<'a> GameBox<'a> {
    // the key of a bundle's contents list in its user data
    const CONTENTS: &'static str = "storage_item_component_content";
    // the key of a bundle's fill weight in its user data
    const WEIGHT: &'static str = "bundle_weight";

    pub fn new(list: NbtList<'a>, heap: MsvcHeap) -> Self {
        Self { list, heap, _owner: None }
    }
}

impl<'a> BoxSource for GameBox<'a> {
    type Stack = OwnedItemStack<'a>;
    type Contents = GameBox<'a>;

    fn len(&self) -> usize {
        self.list.len()
    }

    fn slot(&self, index: usize) -> Option<i32> {
        self.list.compound(index)?.byte("Slot").map(i32::from)
    }

    fn build(&self, index: usize) -> Option<PreviewItem<OwnedItemStack<'a>>> {
        let tag = self.list.compound(index)?;
        let stack = OwnedItemStack::from_tag_in(tag, self.heap)?;
        let (count, glint, bar, bundle, weight) = {
            let base = stack.as_base();
            let bundle = base.type_name().is_some_and(|name| BundleName::is_bundle(&name));
            let weight = if bundle { base.user_data().and_then(|data| data.int(Self::WEIGHT)) } else { None };
            (
                base.count().unwrap_or(0),
                base.is_glint().unwrap_or(false),
                DurabilityBar::new(
                    base.is_damaged().unwrap_or(false),
                    base.damage().unwrap_or(0),
                    base.max_damage().unwrap_or(0),
                ),
                bundle,
                weight,
            )
        };
        Some(PreviewItem { stack, glint, bar, count_text: CountLabel::text(count), bundle, weight })
    }

    #[allow(unsafe_code)]
    fn bundle_contents(&self, index: usize) -> Option<GameBox<'a>> {
        let tag = self.list.compound(index)?;
        let owner = OwnedItemStack::from_tag_in(tag, self.heap)?;
        let list = owner.as_base().user_data()?.list(Self::CONTENTS)?;
        // SAFETY: this extends the borrow of the list's element slice from `owner` to `'a`. The `ItemStack` is a game
        // heap block that does not move when its handle `owner` moves; its user data `CompoundTag`, the `ListTag`
        // and the element buffer are heap objects that stack owns. The returned box keeps `owner` in `_owner`,
        // declared after `list` so it drops last; `list` is private, read only through `&self`, and nothing read
        // from it outlives the box (`build` copies the tag through `ItemStack::fromTag`).
        let list = unsafe { std::mem::transmute::<NbtList<'_>, NbtList<'a>>(list) };
        Some(GameBox { list, heap: self.heap, _owner: Some(owner) })
    }
}
