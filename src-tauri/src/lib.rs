// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod ocr;
mod translation;
mod lang_pack;
mod tray;

use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

use crate::core::config::{load_config, save_config, AppConfig};
use crate::core::database::{Database, HistoryEntry, LanguagePackInfo};
use crate::core::memory::{get_last_activity_elapsed, reclaim_memory};
use crate::core::hotkey::start_hotkey_listener;
use crate::core::inline_type::inject_text_as_keystrokes;
use crate::ocr::{capture_screen_area, run_native_ocr};
use crate::translation::{perform_translation, unload_local_model};
use crate::lang_pack::{download_language_pack, uninstall_language_pack};

// --- IPC Commands ---

#[tauri::command]
async fn translate(
    db: State<'_, Arc<Database>>,
    text: String,
    source: String,
    target: String,
) -> Result<String, String> {
    perform_translation(db.inner().clone(), &text, &source, &target)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_history(db: State<'_, Arc<Database>>, search: Option<String>) -> Result<Vec<HistoryEntry>, String> {
    db.get_history(search).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_favorite(db: State<'_, Arc<Database>>, id: String) -> Result<bool, String> {
    db.toggle_favorite(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_history(db: State<'_, Arc<Database>>, id: String) -> Result<(), String> {
    db.delete_history(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_history(db: State<'_, Arc<Database>>) -> Result<(), String> {
    db.clear_history().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config() -> Result<AppConfig, String> {
    Ok(load_config())
}

#[tauri::command]
fn update_config(config: AppConfig) -> Result<(), String> {
    save_config(&config);
    Ok(())
}

#[tauri::command]
fn get_installed_packs(db: State<'_, Arc<Database>>) -> Result<Vec<LanguagePackInfo>, String> {
    db.get_language_packs().map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_pack(
    app: AppHandle,
    db: State<'_, Arc<Database>>,
    code: String,
    name: String,
) -> Result<(), String> {
    download_language_pack(app, db.inner().clone(), code, name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn uninstall_pack(db: State<'_, Arc<Database>>, code: String) -> Result<(), String> {
    uninstall_language_pack(db.inner().clone(), &code).map_err(|e| e.to_string())
}

#[tauri::command]
async fn ocr_translate(
    db: State<'_, Arc<Database>>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    lang: Option<String>,
    target_lang: String,
) -> Result<String, String> {
    // 1. Capture screen area
    let cropped_png = capture_screen_area(x, y, w, h).map_err(|e| e.to_string())?;

    // 2. Perform native OCR
    let extracted_text = run_native_ocr(&cropped_png, lang.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    if extracted_text.trim().is_empty() {
        return Err("No text detected in selected region".to_string());
    }

    // 3. Perform translation
    let source_lang = lang.unwrap_or_else(|| "Auto".to_string());
    let translated = perform_translation(db.inner().clone(), &extracted_text, &source_lang, &target_lang)
        .await
        .map_err(|e| e.to_string())?;

    // Format return payload as JSON: { original: string, translated: string }
    let response = serde_json::json!({
        "original": extracted_text,
        "translated": translated
    });
    
    Ok(response.to_string())
}

#[tauri::command]
fn inject_typed_translation(text: String) -> Result<(), String> {
    inject_text_as_keystrokes(&text);
    Ok(())
}

#[tauri::command]
fn request_memory_trim() -> Result<(), String> {
    reclaim_memory();
    Ok(())
}

// --- Main Library Entrance ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = Arc::new(Database::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(db)
        .setup(move |app| {
            // 1. Setup tray icon
            let _ = tray::setup_tray(app.handle())?;

            // 2. Start global keyboard listener
            start_hotkey_listener(app.handle().clone());

            // 3. Start memory optimization loop
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_secs(30));
                    
                    let config = load_config();
                    let elapsed = get_last_activity_elapsed();
                    if elapsed.as_secs() >= config.idle_unload_timeout_secs {
                        // Unload model session
                        unload_local_model();
                        // Trim working set RAM
                        reclaim_memory();
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide windows instead of destroying them to preserve state and start instantly
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            translate,
            get_history,
            toggle_favorite,
            delete_history,
            clear_history,
            get_config,
            update_config,
            get_installed_packs,
            download_pack,
            uninstall_pack,
            ocr_translate,
            inject_typed_translation,
            request_memory_trim
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
