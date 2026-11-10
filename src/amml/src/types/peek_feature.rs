use amml_game::{
    CompoundTagRef, ContainerScreenController, EntryId, GameEntries, ItemRenderer, UiRenderContext, UiScene,
};

use crate::game::FontHandleLayout;
use crate::types::PeekEntry;

/// The groups of entries the mod asks for. The four game features are required: with any entry of them missing,
/// peek installs nothing. The font is optional: without it the stack counts are not drawn.
pub struct PeekFeature;

impl PeekFeature {
    /// The font handle and the line length of the stack counts.
    pub const FONT: &'static str = "peek.font";

    /// The wheel hook that picks a bundle. Without it the strip shows the first bundle and the hint is not drawn.
    pub const SCROLL: &'static str = UiScene::FEATURE;

    pub const REQUIRED: [&'static str; 4] =
        [UiRenderContext::FEATURE, ContainerScreenController::FEATURE, ItemRenderer::FEATURE, CompoundTagRef::FEATURE];

    pub fn entries(feature: &str) -> &'static [EntryId] {
        match feature {
            UiRenderContext::FEATURE => &UiRenderContext::ENTRIES,
            ContainerScreenController::FEATURE => &ContainerScreenController::ENTRIES,
            Self::SCROLL => &UiScene::ENTRIES,
            ItemRenderer::FEATURE => &ItemRenderer::ENTRIES,
            CompoundTagRef::FEATURE => &CompoundTagRef::ENTRIES,
            Self::FONT => &[
                PeekEntry::GET_FONT_HANDLE,
                PeekEntry::FONT_HANDLE_DTOR,
                PeekEntry::FONT_HANDLE_SIZE,
                PeekEntry::FONT_GET_LINE_LENGTH,
            ],
            _ => &[],
        }
    }

    /// Whether every entry of the feature resolved and its mirror fits.
    pub fn is_on(entries: &GameEntries, feature: &str) -> bool {
        Self::entries(feature).iter().all(|id| entries.get(*id).is_some()) && Self::misfit(entries, feature).is_none()
    }

    /// The cause when a mirror the feature reads does not fit the build.
    pub fn misfit(entries: &GameEntries, feature: &str) -> Option<String> {
        match feature {
            Self::FONT => FontHandleLayout::misfit(entries),
            _ => None,
        }
    }
}
