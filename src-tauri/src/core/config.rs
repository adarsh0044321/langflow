use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

fn default_replace_selection_directly() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub source_lang: String,
    pub target_lang: String,
    pub translation_mode: String, // "Offline", "Online", "Hybrid"
    pub online_provider: String,  // "DeepL", "Google", "Gemini"
    pub api_key: String,
    pub hotkey_translate: String,
    pub hotkey_ocr: String,
    pub hotkey_typing: String,
    pub inline_typing_enabled: bool,
    #[serde(default = "default_replace_selection_directly")]
    pub replace_selection_directly: bool,
    pub run_on_startup: bool,
    pub idle_unload_timeout_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            source_lang: "Auto".to_string(),
            target_lang: "ja".to_string(),
            translation_mode: "Hybrid".to_string(),
            online_provider: "Google".to_string(),
            api_key: "".to_string(),
            hotkey_translate: "Ctrl+Shift+T".to_string(),
            hotkey_ocr: "Ctrl+Shift+S".to_string(),
            hotkey_typing: "Ctrl+Shift+I".to_string(),
            inline_typing_enabled: false,
            replace_selection_directly: true,
            run_on_startup: false,
            idle_unload_timeout_secs: 180,
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("LangFlow");
    let _ = fs::create_dir_all(&path);
    path
}

pub fn get_config_path() -> PathBuf {
    let mut path = get_config_dir();
    path.push("config.json");
    path
}

pub fn load_config() -> AppConfig {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }
    let default_config = AppConfig::default();
    save_config(&default_config);
    default_config
}

pub fn save_config(config: &AppConfig) {
    let path = get_config_path();
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, content);
    }
}
