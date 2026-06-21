use std::mem;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, KEYBDINPUT, KEYEVENTF_UNICODE, KEYEVENTF_KEYUP};

/// Safely injects text into the active focused window using Win32 SendInput Unicode events.
/// This bypasses copy-paste restrictions and works directly in chat inputs (Discord, games, forms).
pub fn inject_text_as_keystrokes(text: &str) {
    let mut inputs = Vec::new();

    for ch in text.encode_utf16() {
        // Press key (Unicode mode)
        let mut press = INPUT::default();
        press.r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1); // INPUT_KEYBOARD
        press.Anonymous.ki = KEYBDINPUT {
            wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0),
            wScan: ch,
            dwFlags: KEYEVENTF_UNICODE,
            time: 0,
            dwExtraInfo: 0,
        };
        inputs.push(press);

        // Release key (Unicode mode)
        let mut release = INPUT::default();
        release.r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1);
        release.Anonymous.ki = KEYBDINPUT {
            wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0),
            wScan: ch,
            dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: 0,
        };
        inputs.push(release);
    }

    unsafe {
        SendInput(&inputs, mem::size_of::<INPUT>() as i32);
    }
}
