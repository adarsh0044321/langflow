use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use tract_onnx::prelude::*;
use crate::core::config::get_config_dir;

type TractModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

struct ModelCache {
    current_pair: String,
    model: Option<TractModel>,
}

static MODEL_SESSION: Lazy<Mutex<ModelCache>> = Lazy::new(|| {
    Mutex::new(ModelCache {
        current_pair: String::new(),
        model: None,
    })
});

pub fn unload_local_model() {
    let mut cache = MODEL_SESSION.lock().unwrap();
    cache.model = None;
    cache.current_pair = String::new();
}

pub fn get_model_path(source: &str, target: &str) -> PathBuf {
    let mut path = get_config_dir();
    path.push("models");
    path.push(format!("{}-{}", source.to_lowercase(), target.to_lowercase()));
    path.push("model.onnx");
    path
}

pub fn is_model_installed(source: &str, target: &str) -> bool {
    get_model_path(source, target).exists()
}

pub fn translate_local(text: &str, source: &str, target: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let pair = format!("{}-{}", source.to_lowercase(), target.to_lowercase());
    let mut cache = MODEL_SESSION.lock().unwrap();

    // 1. Check if we need to load or switch the model
    if cache.current_pair != pair || cache.model.is_none() {
        let model_file = get_model_path(source, target);
        if !model_file.exists() {
            return Err(format!("Language pack for {} is not installed locally.", pair).into());
        }

        // Load the ONNX model using tract-onnx
        let loaded = tract_onnx::onnx()
            .model_for_path(&model_file)?
            .into_optimized()?
            .into_runnable()?;
            
        cache.model = Some(loaded);
        cache.current_pair = pair;
    }

    let model = cache.model.as_ref().unwrap();

    // 2. Perform tokenization and inference (scaffold logic)
    // neural translation models process input text by tokenizing words into input IDs.
    // Here we run the tract model with mocked tensor input or basic SentencePiece mappings.
    // For compilation completeness, we execute a simple tensor run.
    
    // In a fully populated model, we'd tokenize text -> Vec<i64> of input IDs.
    // Here we mock a basic input tensor shape [1, N] to demonstrate tract graph execution:
    let input_ids = vec![1i64, 2i64, 3i64]; // Mock IDs
    let input_tensor = tract_ndarray::Array2::from_shape_vec((1, 3), input_ids)?;
    let input: Tensor = input_tensor.into();

    // Execute the network
    let result = model.run(tvec![input.into()]);
    
    match result {
        Ok(_outputs) => {
            // Decoded text from outputs (mock translation for scaffold, since full SentencePiece 
            // vocabs are downloaded via language packs)
            let mock_translated = format!("[Local translation of '{}' into {}]", text, target);
            Ok(mock_translated)
        }
        Err(e) => {
            Err(format!("Local inference failed: {}", e).into())
        }
    }
}
