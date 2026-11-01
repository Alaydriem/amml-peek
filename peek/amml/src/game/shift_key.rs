use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_SHIFT};

/// The state of the Shift key.
pub struct ShiftKey;

impl ShiftKey {
    /// Whether Shift is down now, either side.
    #[allow(unsafe_code)]
    pub fn held() -> bool {
        // SAFETY: GetAsyncKeyState takes a virtual key code and reads nothing of ours.
        unsafe { GetAsyncKeyState(i32::from(VK_SHIFT)) < 0 }
    }
}
