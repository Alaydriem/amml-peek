use alaydriem_peek::PeekMod;
use amm_client::test_support::FakeTable;
use amml_api::amml_abi::{
    ContainerSlotHover, ContainerSlotUnhover, ItemHovertext, KeyInput, ModsLoaded, ScreenClosing, ScreenRender,
    UiScroll,
};
use amml_api::testing::{FakeEntry, FakeGame, FakeItem, FakeScreen, FakeSlots};
use amml_api::{Area, Collection, Point, ScreenId, ScreenKind, Size, SlotAt, ViewId};

pub const SCREEN: ScreenId = ScreenId(7);
pub const VIEW: ViewId = ViewId(70);

// the peek mod enabled on a fake game that serves every row of its `USES`
pub struct PeekRig {
    pub game: FakeGame,
}

impl PeekRig {
    // the mod manager is provided, so the link connects at enable
    pub fn with_manager() -> Self {
        Self::build(true)
    }

    // the mod manager is not installed
    pub fn without_manager() -> Self {
        Self::build(false)
    }

    // a fake game that serves every row of `USES`, with the mod not loaded yet
    pub fn offered() -> FakeGame {
        let game = FakeTable::game();
        game.offer::<ScreenRender>();
        game.offer::<ContainerSlotHover>();
        game.offer::<ContainerSlotUnhover>();
        game.offer::<ScreenClosing>();
        game.offer::<ModsLoaded>();
        game.offer::<UiScroll>();
        game.offer::<KeyInput>();
        game.offer::<ItemHovertext>();
        game.offer_area(Area::UI_DRAW.at(2));
        game.offer_area(Area::UI_SCREEN.at(2));
        game.offer_area(Area::INVENTORY_READ.at(4));
        game.offer_area(Area::UI_TEXT);
        game
    }

    fn build(manager: bool) -> Self {
        let game = Self::offered();
        if manager {
            FakeTable::provide(&game.ctx()).unwrap();
        }
        game.load::<PeekMod>().unwrap();
        game.enable::<PeekMod>().unwrap();
        game.open_gate();
        Self { game }
    }

    // a chest whose container slots are `slots`, with the pointer at the middle of a 400 by 300 view
    pub fn chest(slots: Vec<Option<FakeItem>>) -> FakeScreen {
        FakeScreen {
            kind: ScreenKind::SmallChest,
            size: Some(Size { width: 400, height: 300 }),
            pointer: Some(Point { x: 200.0, y: 150.0 }),
            can_draw: true,
            slots: FakeSlots { container: slots, hotbar: Vec::new(), inventory: Vec::new() },
            ..FakeScreen::default()
        }
    }

    // a shulker box with these entries; a filled box has user data, so no identity
    pub fn shulker(entries: Vec<FakeEntry>) -> FakeItem {
        FakeItem {
            type_name: "minecraft:purple_shulker_box",
            count: 1,
            aux: Some(0),
            contents: Some(Box::leak(entries.into_boxed_slice())),
            ..FakeItem::EMPTY
        }
    }

    pub fn item(slot: u8, type_name: &'static str, count: u8) -> FakeEntry {
        FakeEntry { slot, item: FakeItem { type_name, count, ..FakeItem::EMPTY }, bundle: None, bundle_weight: None }
    }

    pub fn bundle(slot: u8, weight: i32, contents: Vec<FakeEntry>) -> FakeEntry {
        FakeEntry {
            slot,
            item: FakeItem { type_name: "minecraft:bundle", count: 1, ..FakeItem::EMPTY },
            bundle: Some(Box::leak(contents.into_boxed_slice())),
            bundle_weight: Some(weight),
        }
    }

    pub fn container(index: i32) -> SlotAt {
        SlotAt { collection: Collection::Container, index }
    }

    pub fn hover(&self, index: i32) {
        self.game.raise_hover(SCREEN, Self::container(index));
    }

    pub fn unhover(&self, index: i32) {
        self.game.raise_unhover(SCREEN, Self::container(index));
    }

    pub fn render(&self, screen: &FakeScreen) {
        self.game.raise_screen_render(SCREEN, VIEW, screen);
    }

    pub fn shift(&self, down: bool) {
        self.game.real_key(0xA0, down);
    }

    // true when the log holds `line` (level, message and fields as the fake writes them)
    pub fn logged(&self, line: &str) -> bool {
        self.game.logs().iter().any(|logged| logged == line)
    }

    pub fn logged_count(&self, start: &str) -> usize {
        self.game.logs().iter().filter(|logged| logged.starts_with(start)).count()
    }

    // the item draws of the fake, `item <type name> <x> <y> <flags>`
    pub fn item_draws(&self) -> Vec<String> {
        self.game.draws().into_iter().filter(|draw| draw.starts_with("item ")).collect()
    }
}
