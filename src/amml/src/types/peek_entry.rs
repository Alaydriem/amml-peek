use amml_game::EntryId;

/// The entries of the stack count text; the other entries are shared ones in amml-game.
pub struct PeekEntry;

impl PeekEntry {
    pub const GET_FONT_HANDLE: EntryId = EntryId::new("ClientInstance.getFontHandle");
    pub const FONT_HANDLE_DTOR: EntryId = EntryId::new("FontHandle.dtor");
    pub const FONT_HANDLE_SIZE: EntryId = EntryId::new("FontHandle.size");
    pub const FONT_GET_LINE_LENGTH: EntryId = EntryId::new("Font.getLineLength");
}
