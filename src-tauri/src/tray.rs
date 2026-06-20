use tauri::{AppHandle, Manager};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define menu items
    let open_item = MenuItem::with_id(app, "open", "Open Translator", true, None::<&str>)?;
    let ocr_item = MenuItem::with_id(app, "ocr", "Screenshot Translate", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings & Languages", true, None::<&str>)?;
    let exit_item = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;

    // 2. Build the menu list
    let menu = Menu::with_items(
        app,
        &[&open_item, &ocr_item, &settings_item, &exit_item],
    )?;

    // 3. Build and launch tray icon using embedded bytes to ensure it never fails
    let icon_bytes = include_bytes!("../icons/32x32.png");
    let decoded = image::load_from_memory(icon_bytes)?;
    let rgba = decoded.to_rgba8();
    let width = decoded.width();
    let height = decoded.height();
    let icon = tauri::image::Image::new_owned(rgba.into_raw(), width, height);

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .on_menu_event(|app_handle, event| {
            match event.id.as_ref() {
                "open" => {
                    if let Some(w) = app_handle.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                "ocr" => {
                    if let Some(w) = app_handle.get_webview_window("screenshot_overlay") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                "settings" => {
                    if let Some(w) = app_handle.get_webview_window("settings") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                "exit" => {
                    std::process::exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick { button: tauri::tray::MouseButton::Left, .. } = event {
                let app_handle = tray.app_handle();
                if let Some(w) = app_handle.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
