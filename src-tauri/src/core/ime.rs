use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::mem;
use once_cell::sync::{Lazy, OnceCell};
use tauri::{AppHandle, Manager};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, GetMessageW, MSG, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_SYSKEYDOWN, KBDLLHOOKSTRUCT, HHOOK
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardState, ToUnicode, SendInput, INPUT, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY
};

use crate::core::config::load_config;
use crate::core::database::Database;
use crate::translation::perform_translation;

static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

struct ActiveInputState {
    original_sentence: String,
    current_word: String,
    translated_lengths: Vec<usize>,
    is_enabled: bool,
}

static IME_STATE: Lazy<Mutex<ActiveInputState>> = Lazy::new(|| Mutex::new(ActiveInputState {
    original_sentence: String::new(),
    current_word: String::new(),
    translated_lengths: Vec::new(),
    is_enabled: false,
}));

static mut HOOK_HANDLE: Option<HHOOK> = None;

pub fn init_ime_hook(app: AppHandle) {
    let _ = APP_HANDLE.set(app.clone());
    
    // Set initial enabled state from loaded config
    let config = load_config();
    {
        let mut state = IME_STATE.lock().unwrap();
        state.is_enabled = config.inline_typing_enabled;
    }

    // Spawn a dedicated thread for the Win32 hook and message loop
    thread::spawn(move || {
        unsafe {
            let h_instance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                .unwrap_or_default();
            
            let hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_proc),
                h_instance,
                0,
            );

            if let Ok(h) = hook {
                HOOK_HANDLE = Some(h);
                
                // Low-level hooks require a standard Win32 message loop
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
                    // process messages
                }

                let _ = UnhookWindowsHookEx(h);
            }
        }
    });
}

pub fn set_ime_enabled(enabled: bool) {
    let mut state = IME_STATE.lock().unwrap();
    state.is_enabled = enabled;
    
    // Clear buffers on toggle
    state.current_word.clear();
    state.original_sentence.clear();
    state.translated_lengths.clear();
}

pub fn get_app_handle() -> AppHandle {
    APP_HANDLE.get().unwrap().clone()
}

unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let is_keydown = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
        if is_keydown {
            let kbd_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            
            // Ignore injected inputs (flags.0 bit 4 is LLKHF_INJECTED) to avoid loops and self-triggering
            let is_injected = (kbd_struct.flags.0 & 0x10) != 0;
            if !is_injected {
                let vk_code = kbd_struct.vkCode;
                if process_key_event(vk_code, kbd_struct.scanCode) {
                    // Swallow the key event
                    return LRESULT(1);
                }
            }
        }
    }
    CallNextHookEx(HOOK_HANDLE.unwrap_or_default(), code, wparam, lparam)
}

fn process_key_event(vk_code: u32, scan_code: u32) -> bool {
    let mut state = IME_STATE.lock().unwrap();
    if !state.is_enabled {
        return false;
    }

    if vk_code == 0x08 { // VK_BACK (Backspace)
        if !state.current_word.is_empty() {
            state.current_word.pop();
            state.original_sentence.pop();
        } else if !state.original_sentence.is_empty() {
            state.original_sentence.pop();
        } else {
            state.translated_lengths.clear();
        }
        return false;
    }

    if vk_code == 0x1B { // VK_ESCAPE
        state.current_word.clear();
        state.original_sentence.clear();
        state.translated_lengths.clear();
        return false;
    }

    if vk_code == 0x20 { // VK_SPACE
        let word = state.current_word.clone();
        state.current_word.clear();
        state.original_sentence.push(' ');

        if !word.trim().is_empty() {
            trigger_word_translation(word);
        }
        return false;
    }

    // VK_RETURN (Enter = 0x0D), OEM Period ('.' = 0xBE), OEM Question ('?' = 0xBF), OEM 1 ('!' = 0x31)
    if vk_code == 0x0D {
        // Handle Enter: we swallow Enter, translate sentence, replace, and then simulate Enter key press!
        let sentence = state.original_sentence.clone();
        let last_word_len = state.current_word.len();
        
        state.current_word.clear();
        state.original_sentence.clear();
        let lengths = state.translated_lengths.clone();
        state.translated_lengths.clear();

        if !sentence.trim().is_empty() {
            trigger_sentence_translation(sentence, lengths, last_word_len, 0, true);
            return true; // Swallow original Enter press
        }
        return false;
    }

    if let Some(ch) = unsafe { vk_to_char(vk_code, scan_code) } {
        if ch.is_alphanumeric() || ch == '\'' || ch == '-' || ch == ',' {
            state.current_word.push(ch);
            state.original_sentence.push(ch);
        } else if ch == '.' || ch == '?' || ch == '!' {
            let sentence = state.original_sentence.clone();
            let last_word_len = state.current_word.len();
            
            state.current_word.clear();
            state.original_sentence.clear();
            let lengths = state.translated_lengths.clone();
            state.translated_lengths.clear();

            if !sentence.trim().is_empty() {
                // Period/Question marks print to screen normally, so we delete them as part of trigger_char_len
                trigger_sentence_translation(sentence, lengths, last_word_len, 1, false);
            }
        }
    }

    false
}

fn trigger_word_translation(word: String) {
    let app = get_app_handle();
    tauri::async_runtime::spawn(async move {
        let db = app.state::<Arc<Database>>();
        let config = load_config();
        
        // Brief sleep to let Space pass through to target window
        tokio::time::sleep(Duration::from_millis(30)).await;

        if let Ok(translated) = perform_translation(
            db.inner().clone(),
            &word,
            &config.source_lang,
            &config.target_lang,
        ).await {
            // Delete word + space
            let backspaces = word.chars().count() + 1;
            unsafe {
                simulate_backspaces(backspaces);
            }
            tokio::time::sleep(Duration::from_millis(15)).await;

            let replacement = format!("{} ", translated);
            {
                let mut state = IME_STATE.lock().unwrap();
                state.translated_lengths.push(replacement.chars().count());
            }

            crate::core::inline_type::inject_text_as_keystrokes(&replacement);
        }
    });
}

fn trigger_sentence_translation(
    sentence: String,
    lengths_to_erase: Vec<usize>,
    last_word_len: usize,
    trigger_char_len: usize,
    send_enter_after: bool,
) {
    let app = get_app_handle();
    tauri::async_runtime::spawn(async move {
        let db = app.state::<Arc<Database>>();
        let config = load_config();

        tokio::time::sleep(Duration::from_millis(30)).await;

        if let Ok(translated) = perform_translation(
            db.inner().clone(),
            &sentence,
            &config.source_lang,
            &config.target_lang,
        ).await {
            let sum_injected: usize = lengths_to_erase.iter().sum();
            // Erase: word translations + last untranslated word + trigger character (if typed)
            let total_backspaces = sum_injected + last_word_len + trigger_char_len;

            if total_backspaces > 0 {
                unsafe {
                    simulate_backspaces(total_backspaces);
                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;

            // Type translated sentence
            crate::core::inline_type::inject_text_as_keystrokes(&translated);
            
            if send_enter_after {
                tokio::time::sleep(Duration::from_millis(20)).await;
                unsafe {
                    simulate_enter_key();
                }
            }
        } else {
            // If translation failed, restore Enter if we swallowed it
            if send_enter_after {
                unsafe {
                    simulate_enter_key();
                }
            }
        }
    });
}

unsafe fn vk_to_char(vk: u32, scan_code: u32) -> Option<char> {
    let mut keyboard_state = [0u8; 256];
    let _ = GetKeyboardState(&mut keyboard_state);

    // Manually read keyboard modifier states (Shift, Caps Lock) for accurate translation
    let is_shift = (windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(0x10) as u16 & 0x8000) != 0; // VK_SHIFT
    let is_caps = (windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(0x14) as u16 & 0x0001) != 0; // VK_CAPITAL (Caps Lock)
    
    if is_shift {
        keyboard_state[0x10] = 0x80;
    }
    if is_caps {
        keyboard_state[0x14] = 0x01;
    }

    let mut buf = [0u16; 8];
    let len = ToUnicode(
        vk,
        scan_code,
        Some(&keyboard_state),
        &mut buf,
        0,
    );

    if len > 0 {
        char::from_u32(buf[0] as u32)
    } else {
        // Fallback translation mapping for standard virtual keys when running on non-GUI background threads
        match vk {
            // A-Z
            0x41..=0x5A => {
                let base = (vk - 0x41) as u8;
                let uppercase = is_shift ^ is_caps;
                if uppercase {
                    Some((b'A' + base) as char)
                } else {
                    Some((b'a' + base) as char)
                }
            }
            // Numeric Row
            0x30..=0x39 => {
                let base = (vk - 0x30) as u8;
                if is_shift {
                    match base {
                        1 => Some('!'),
                        2 => Some('@'),
                        3 => Some('#'),
                        4 => Some('$'),
                        5 => Some('%'),
                        6 => Some('^'),
                        7 => Some('&'),
                        8 => Some('*'),
                        9 => Some('('),
                        0 => Some(')'),
                        _ => None,
                    }
                } else {
                    Some((b'0' + base) as char)
                }
            }
            // Numpad Numbers
            0x60..=0x69 => {
                let base = (vk - 0x60) as u8;
                Some((b'0' + base) as char)
            }
            // Punctuation
            0xBC => Some(if is_shift { '<' } else { ',' }), // OEM_COMMA
            0xBD => Some(if is_shift { '_' } else { '-' }), // OEM_MINUS
            0xBE => Some(if is_shift { '>' } else { '.' }), // OEM_PERIOD
            0xBF => Some(if is_shift { '?' } else { '/' }), // OEM_2
            0xDE => Some(if is_shift { '"' } else { '\'' }), // OEM_7
            _ => None,
        }
    }
}

unsafe fn simulate_backspaces(count: usize) {
    if count == 0 { return; }
    let mut inputs = Vec::new();
    for _ in 0..count {
        let mut press = INPUT::default();
        press.r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1);
        press.Anonymous.ki = KEYBDINPUT {
            wVk: VIRTUAL_KEY(0x08), // VK_BACK
            wScan: 0,
            dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
            time: 0,
            dwExtraInfo: 0,
        };
        inputs.push(press);

        let mut release = INPUT::default();
        release.r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1);
        release.Anonymous.ki = KEYBDINPUT {
            wVk: VIRTUAL_KEY(0x08),
            wScan: 0,
            dwFlags: KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: 0,
        };
        inputs.push(release);
    }
    SendInput(&inputs, mem::size_of::<INPUT>() as i32);
}

unsafe fn simulate_enter_key() {
    let mut inputs = [INPUT::default(); 2];
    
    // Press Enter
    inputs[0].r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1);
    inputs[0].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0x0D), // VK_RETURN
        wScan: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
        time: 0,
        dwExtraInfo: 0,
    };

    // Release Enter
    inputs[1].r#type = windows::Win32::UI::Input::KeyboardAndMouse::INPUT_TYPE(1);
    inputs[1].Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(0x0D),
        wScan: 0,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: 0,
    };

    SendInput(&inputs, mem::size_of::<INPUT>() as i32);
}
