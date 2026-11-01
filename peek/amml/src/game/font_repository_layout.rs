use std::ffi::c_void;

use amml_msvc::{MsvcVec, SharedPtr};

/// Layout mirror of the start of the game's `FontRepository`, reached only through the pointer in a `FontHandle`.
#[repr(C)]
pub struct FontRepositoryLayout {
    _bases: [u8; 0x38],                       // 0x00 AppPlatformListener, EnableNonOwnerReferences
    _is_initialized: bool,                    // 0x38 mIsInitialized
    loaded_fonts: MsvcVec<SharedPtr<c_void>>, // 0x40 mLoadedFonts
}

impl FontRepositoryLayout {
    /// The loaded fonts; a font id is an index into them.
    pub fn fonts(&self) -> &[SharedPtr<c_void>] {
        self.loaded_fonts.as_slice()
    }
}
