use std::ffi::c_void;
use std::mem::MaybeUninit;

use amml_game::{GameEntries, RawObject};

use crate::game::FontHandleLayout;
use crate::types::PeekEntry;

/// The default `FontHandle` of the client, held while text is queued. Dropping it runs the game's destructor, so it
/// must live until `flush_text` has run. Client main thread.
pub struct FontScope {
    handle: Box<MaybeUninit<FontHandleLayout>>,
    destroy: usize,
    line_length_slot: u64,
}

impl FontScope {
    // the font size the widths are measured at, the size the counts are drawn at
    const FONT_SIZE: f32 = 1.0;

    /// Asks `client` for its font handle. `None` when an entry is off, the size entry is not the mirror size, or a
    /// pointer is null.
    ///
    /// # Safety
    /// `client` must be a live `ClientInstance` of the game build `entries` was resolved for.
    #[allow(unsafe_code)]
    pub unsafe fn open(entries: &GameEntries, client: *mut c_void) -> Option<Self> {
        let slot = entries.get(PeekEntry::GET_FONT_HANDLE)?;
        let destroy = usize::try_from(entries.get(PeekEntry::FONT_HANDLE_DTOR)?).ok()?;
        let line_length_slot = entries.get(PeekEntry::FONT_GET_LINE_LENGTH)?;
        // every check is made before the handle exists, so that no early return leaves a handle undestroyed
        if destroy == 0 || client.is_null() || !FontHandleLayout::fits(entries) {
            return None;
        }
        // SAFETY: the caller guarantees a live client, whose first qword is its vtable, and the entry is the slot of
        // `getFontHandle` in it.
        let function = unsafe { RawObject::slot(client, slot) }?;
        // SAFETY: the slot holds `FontHandle ClientInstance::getFontHandle() const`: this in rcx, the hidden return
        // buffer in rdx.
        let get_handle: unsafe extern "C" fn(*mut c_void, *mut FontHandleLayout) -> *mut c_void =
            unsafe { std::mem::transmute(function) };
        let mut handle = Box::new(MaybeUninit::<FontHandleLayout>::zeroed());
        // SAFETY: the buffer is a FontHandle, whose size was checked against the build, and the client is live.
        unsafe { get_handle(client, handle.as_mut_ptr()) };
        Some(Self { handle, destroy, line_length_slot })
    }

    /// The `Font*` of the handle; valid while `self` lives. `None` when the picked pointer is null.
    #[allow(unsafe_code)]
    pub fn font(&self) -> Option<*mut c_void> {
        // SAFETY: `open` had the game fill the handle just now, and its repository is the game's.
        unsafe { (*self.handle.as_ptr()).font() }
    }

    /// The width of `text` in the font, through `Font::getLineLength`. `None` when the font or the call is missing.
    #[allow(unsafe_code)]
    pub fn line_width(&self, text: &str) -> Option<i32> {
        let font = self.font()?;
        // SAFETY: the font is live while the handle is, and the entry is the slot of `getLineLength` in its vtable.
        let function = unsafe { RawObject::slot(font, self.line_length_slot) }?;
        // SAFETY: the slot holds `int Font::getLineLength(std::string_view, float, bool)`: this in rcx, a pointer to
        // the string_view copy in rdx, the size in xmm2, the bool in r9.
        let measure: unsafe extern "C" fn(*mut c_void, *const amml_game::StringViewLayout, f32, u8) -> i32 =
            unsafe { std::mem::transmute(function) };
        let view = amml_game::StringViewLayout::of(text);
        // SAFETY: the view lives for the call, and the font is live.
        Some(unsafe { measure(font, &raw const view, Self::FONT_SIZE, 0) })
    }
}

impl Drop for FontScope {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: the entry is the address of the non-deleting `FontHandle::~FontHandle()`, which takes this in rcx.
        let destroy: unsafe extern "C" fn(*mut FontHandleLayout) =
            unsafe { std::mem::transmute(self.destroy as *const c_void) };
        // SAFETY: the handle was filled by the game and is destroyed exactly once.
        unsafe { destroy(self.handle.as_mut_ptr()) };
    }
}
