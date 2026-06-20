use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::mem;
use arboard::Clipboard;
use tauri::{AppHandle, Manager, WebviewWindow, Emitter};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, SendInput, INPUT, INPUT_0, KEYBDINPUT, KEYEVENTF_KEYUP, MOD_CONTROL, MOD_SHIFT, VIRTUAL_KEY, VK_C, VK_I, VK_S, VK_T
};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetMessageW, MSG, WM_HOTKEY};
use windows::Win32::Foundation::{HWND, POINT};

const HOTKEY_TRANSLATE_ID: i32 = 1001;
const HOTKEY_OCR_ID: i32 = 1002;
const HOTKEY_TYPING_ID: i32 = 1003;

pub fn start_hotkey_listener(app_handle: AppHandle) {
    thread::spawn(move || {
        unsafe {
            // Force creation of the thread's message queue so hotkeys register correctly
            let mut init_msg = MSG::default();
            let _ = windows::Win32::UI::WindowsAndMessaging::PeekMessageW(
                &mut init_msg,
                HWND(0),
                0,
                0,
                windows::Win32::UI::WindowsAndMessaging::PM_NOREMOVE,
            );

            // Register hotkeys globally:
            // Ctrl + Shift + T (Translate Highlight)
            let _ = RegisterHotKey(
                HWND(0),
                HOTKEY_TRANSLATE_ID,
                MOD_CONTROL | MOD_SHIFT,
                VK_T.0 as u32,
            );

            // Ctrl + Shift + S (Screenshot OCR)
            let _ = RegisterHotKey(
                HWND(0),
                HOTKEY_OCR_ID,
                MOD_CONTROL | MOD_SHIFT,
                VK_S.0 as u32,
            );

            // Ctrl + Shift + I (Toggle Typing Assistant)
            let _ = RegisterHotKey(
                HWND(0),
                HOTKEY_TYPING_ID,
                MOD_CONTROL | MOD_SHIFT,
                VK_I.0 as u32,
            );

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
                if msg.message == WM_HOTKEY {
                    let hotkey_id = msg.wParam.0 as i32;
                    let handle = app_handle.clone();
                    
                    match hotkey_id {
                        HOTKEY_TRANSLATE_ID => {
                            // Run clipboard translation
                            tokio::spawn(async move {
                                let _ = process_highlight_translation(handle).await;
                            });
                        }
                        HOTKEY_OCR_ID => {
                            // Open screenshot overlay window
                            let app_clone = app_handle.clone();
                            let _ = app_handle.run_on_main_thread(move || {
                                if let Some(window) = app_clone.get_webview_window("screenshot_overlay") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            });
                        }
                        HOTKEY_TYPING_ID => {
                            // Run inline typing replacement translation
                            let handle = app_handle.clone();
                            tokio::spawn(async move {
                                let _ = process_inline_typing_translation(handle).await;
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    });
}

/// Simulates Ctrl+C keystroke to copy current highlight, reads clipboard,
/// queries mouse cursor position, moves popup window, and triggers translation.
async fn process_highlight_translation(app: AppHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Synthesize Ctrl+C key presses
    unsafe {
        simulate_copy_keys();
    }

    // 2. Wait briefly for OS clipboard state to populate
    tokio::time::sleep(Duration::from_millis(120)).await;

    // 3. Read clipboard contents
    let mut clipboard = Clipboard::new()?;
    let selected_text = match clipboard.get_text() {
        Ok(text) if !text.trim().is_empty() => text,
        _ => return Ok(()), // No text selected
    };

    // 4. Query mouse cursor position
    let mut point = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut point);
    }

    // 5. Setup popup window coordinates and invoke translate event
    let app_clone = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(popup) = app_clone.get_webview_window("floating_popup") {
            // Position popup slightly offset from cursor
            let _ = popup.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: point.x + 10,
                y: point.y + 10,
            }));
            let _ = popup.show();
            let _ = popup.set_focus();
            let _ = popup.emit("translate-highlight", selected_text);
        }
    });

    Ok(())
}

async fn process_inline_typing_translation(app: AppHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Read original clipboard text so we don't destroy user's copy-paste buffer
    let original_clipboard_text = {
        let mut cb = Clipboard::new()?;
        cb.get_text().ok()
    };

    // 2. Synthesize Shift + Home key press to highlight the typed text
    unsafe {
        simulate_select_line_keys();
    }
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 3. Synthesize Ctrl+C key presses to copy the highlighted text
    unsafe {
        simulate_copy_keys();
    }
    tokio::time::sleep(Duration::from_millis(150)).await;

    // 4. Read highlighted text from clipboard
    let selected_text = {
        let mut cb = Clipboard::new()?;
        match cb.get_text() {
            Ok(text) if !text.trim().is_empty() => text,
            _ => return Ok(()), // Nothing selected/copied
        }
    };

    // 5. Restore user's original clipboard
    if let Some(orig) = original_clipboard_text {
        let mut cb = Clipboard::new()?;
        let _ = cb.set_text(orig);
    }

    // 6. Retrieve translation config and DB state
    let config = crate::core::config::load_config();
    let db = app.state::<Arc<crate::core::database::Database>>();

    // 7. Perform translation
    let translated = crate::translation::perform_translation(
        db.inner().clone(),
        &selected_text,
        &config.source_lang,
        &config.target_lang,
    ).await?;

    // 8. Synthesize Backspace to delete the highlighted original text
    unsafe {
        simulate_backspace_key();
    }
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 9. Type the translated text back character by character
    crate::core::inline_type::inject_text_as_keystrokes(&translated);

    Ok(())
}

unsafe fn simulate_copy_keys() {
    let mut inputs = [INPUT::default(); 4];

    // Press Ctrl
    inputs[0].r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1); // INPUT_KEYBOARD
    inputs[0].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0x11), // VK_CONTROL
        wScan: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
        time: 0,
        dwExtraInfo: 0,
    };

    // Press C
    inputs[1].r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1);
    inputs[1].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0x43), // VK_C
        wScan: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
        time: 0,
        dwExtraInfo: 0,
    };

    // Release C
    inputs[2].r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1);
    inputs[2].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0x43),
        wScan: 0,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: 0,
    };

    // Release Ctrl
    inputs[3].r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1);
    inputs[3].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0x11),
        wScan: 0,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: 0,
    };

    SendInput(&inputs, mem::size_of::<INPUT>() as i32);
}

unsafe fn simulate_select_line_keys() {
    let mut inputs = [INPUT::default(); 4];

    // Press Shift
    inputs[0].r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1); // INPUT_KEYBOARD
    inputs[0].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0x10), // VK_SHIFT
        wScan: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
        time: 0,
        dwExtraInfo: 0,
    };

    // Press Home
    inputs[1].r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1);
    inputs[1].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0x24), // VK_HOME
        wScan: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
        time: 0,
        dwExtraInfo: 0,
    };

    // Release Home
    inputs[2].r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1);
    inputs[2].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0x24),
        wScan: 0,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: 0,
    };

    // Release Shift
    inputs[3].r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1);
    inputs[3].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0x10),
        wScan: 0,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: 0,
    };

    SendInput(&inputs, mem::size_of::<INPUT>() as i32);
}

unsafe fn simulate_backspace_key() {
    let mut inputs = [INPUT::default(); 2];

    // Press Backspace
    inputs[0].r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1); // INPUT_KEYBOARD
    inputs[0].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0x08), // VK_BACK
        wScan: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
        time: 0,
        dwExtraInfo: 0,
    };

    // Release Backspace
    inputs[1].r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1);
    inputs[1].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0x08),
        wScan: 0,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: 0,
    };

    SendInput(&inputs, mem::size_of::<INPUT>() as i32);
}
