use crate::core::config::{load_config, AppConfig};
use crate::core::database::Database;
use crate::core::memory::update_activity;
use crate::translation::local_onnx::{is_model_installed, translate_local};
use crate::translation::online_api::{translate_deepl, translate_gemini, translate_google};
use std::sync::Arc;

pub async fn perform_translation(
    db: Arc<Database>,
    text: &str,
    source: &str,
    target: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let trimmed_text = text.trim();
    if trimmed_text.is_empty() {
        return Ok(String::new());
    }

    // Mark activity for memory unload scheduler
    update_activity();

    // 1. Check SQLite Cache
    if let Some(cached) = db.get_cache(source, target, trimmed_text) {
        return Ok(cached);
    }

    let config: AppConfig = load_config();
    let mode = config.translation_mode.as_str();

    let result = match mode {
        "Offline" => {
            translate_local(trimmed_text, source, target)
        }
        "Online" => {
            run_online_translation(&config, trimmed_text, source, target).await
        }
        _ => {
            // Hybrid Mode
            if is_model_installed(source, target) {
                match translate_local(trimmed_text, source, target) {
                    Ok(translated) => Ok(translated),
                    Err(e) => {
                        eprintln!("Local translation failed in Hybrid Mode: {}. Falling back to online.", e);
                        run_online_translation(&config, trimmed_text, source, target).await
                    }
                }
            } else {
                run_online_translation(&config, trimmed_text, source, target).await
            }
        }
    };

    // 2. Cache successful translation and log to history
    if let Ok(ref translated) = result {
        db.set_cache(source, target, trimmed_text, translated);
        let _ = db.add_history(source, target, trimmed_text, translated);
    }

    result
}

async fn run_online_translation(
    config: &AppConfig,
    text: &str,
    source: &str,
    target: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    match config.online_provider.as_str() {
        "DeepL" if !config.api_key.is_empty() => {
            translate_deepl(text, &config.api_key, target).await
        }
        "Gemini" if !config.api_key.is_empty() => {
            translate_gemini(text, &config.api_key, target).await
        }
        _ => {
            // Fallback to Google free translate
            translate_google(text, source, target).await
        }
    }
}
