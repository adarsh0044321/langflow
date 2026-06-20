use serde_json::Value;

pub async fn translate_google(text: &str, _source: &str, target: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t&q={}",
        target,
        urlencoding::encode(text)
    );

    let res = client.get(&url)
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await?
        .json::<Value>()
        .await?;

    // The GoogleTranslate format is a nested array:
    // [[[ "translated_text", "original_text", ... ], ...]]
    let mut translated = String::new();
    if let Some(sentences) = res.get(0).and_then(|v| v.as_array()) {
        for sentence in sentences {
            if let Some(parts) = sentence.as_array() {
                if let Some(part) = parts.get(0).and_then(|v| v.as_str()) {
                    translated.push_str(part);
                }
            }
        }
    }

    if translated.is_empty() {
        return Err("Empty translation response from Google".into());
    }

    Ok(translated)
}

pub async fn translate_deepl(text: &str, api_key: &str, target: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let is_free_key = api_key.ends_with(":fx");
    let host = if is_free_key {
        "api-free.deepl.com"
    } else {
        "api.deepl.com"
    };

    let url = format!("https://{}/v2/translate", host);
    
    let response = client.post(&url)
        .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
        .timeout(std::time::Duration::from_secs(8))
        .form(&[
            ("text", text),
            ("target_lang", target),
        ])
        .send()
        .await?
        .json::<Value>()
        .await?;

    if let Some(translations) = response.get("translations").and_then(|v| v.as_array()) {
        if let Some(first) = translations.get(0) {
            if let Some(translated) = first.get("text").and_then(|v| v.as_str()) {
                return Ok(translated.to_string());
            }
        }
    }

    Err("Invalid response from DeepL".into())
}

pub async fn translate_gemini(text: &str, api_key: &str, target: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let system_prompt = format!(
        "You are an expert translator. Translate the following text to language code '{}'. \
         Provide ONLY the translation, preserve formatting and styling if possible. Do NOT write any chat or introduction.",
        target
    );

    let payload = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": text
            }]
        }],
        "systemInstruction": {
            "parts": [{
                "text": system_prompt
            }]
        },
        "generationConfig": {
            "temperature": 0.3,
            "maxOutputTokens": 2048
        }
    });

    let res = client.post(&url)
        .timeout(std::time::Duration::from_secs(10))
        .json(&payload)
        .send()
        .await?
        .json::<Value>()
        .await?;

    if let Some(candidates) = res.get("candidates").and_then(|v| v.as_array()) {
        if let Some(first_cand) = candidates.get(0) {
            if let Some(parts) = first_cand.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                if let Some(text_part) = parts.get(0).and_then(|tp| tp.get("text")).and_then(|t| t.as_str()) {
                    return Ok(text_part.trim().to_string());
                }
            }
        }
    }

    Err("Invalid response from Gemini API".into())
}
