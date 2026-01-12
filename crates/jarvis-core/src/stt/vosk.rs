use once_cell::sync::OnceCell;
use vosk::{DecodingState, Model, Recognizer};

use std::sync::Mutex;

// use crate::config::VOSK_MODEL_PATH;
use crate::{stt::vosk_models, i18n, config};
use crate::DB;

static MODEL: OnceCell<Model> = OnceCell::new();
static WAKE_RECOGNIZER: OnceCell<Mutex<Recognizer>> = OnceCell::new();
static SPEECH_RECOGNIZER: OnceCell<Mutex<Recognizer>> = OnceCell::new();

pub fn init_vosk() -> Result<(), String> {
    if MODEL.get().is_some() {
        return Ok(());
    } // already initialized

    let model_path = get_configured_model_path()?;
    info!("Loading Vosk model from: {}", model_path.display());

    let model = Model::new(model_path.to_str().unwrap())
        .ok_or_else(|| format!("Failed to load Vosk model from: {}", model_path.display()))?;

    // language-specific wake grammar
    let lang = i18n::get_language();
    let wake_grammar = config::get_wake_grammar(&lang);
    info!("Wake grammar for '{}': {:?}", lang, wake_grammar);

    //let mut recognizer = Recognizer::new(&model, 16000.0)
    //    .ok_or("Failed to create Vosk recognizer")?;
    let mut wake_recognizer = Recognizer::new_with_grammar(&model, 16000.0, wake_grammar)
        .ok_or("Failed to create wake word recognizer")?;

    wake_recognizer.set_max_alternatives(1); // required for confidence check later on

    let mut speech_recognizer = Recognizer::new(&model, 16000.0)
        .ok_or("Failed to create speech recognizer")?;

    speech_recognizer.set_max_alternatives(config::VOSK_SPEECH_RECOGNIZER_MAX_ALTERNATIVES);
    speech_recognizer.set_words(config::VOSK_SPEECH_RECOGNIZER_WORDS);
    speech_recognizer.set_partial_words(config::VOSK_SPEECH_PARTIAL_WORDS);

    MODEL.set(model).map_err(|_| "Model already set")?;
    WAKE_RECOGNIZER.set(Mutex::new(wake_recognizer)).map_err(|_| "Wake recognizer already set")?;
    SPEECH_RECOGNIZER.set(Mutex::new(speech_recognizer)).map_err(|_| "Speech recognizer already set")?;

    Ok(())
}


pub fn recognize_wake_word(data: &[i16]) -> Option<(String, f32)> {
    let mut recognizer = WAKE_RECOGNIZER.get()?.lock().unwrap();
    
    match recognizer.accept_waveform(data) {
        Ok(DecodingState::Running) => {
            // partials don't have confidence, skip them
            None
        }
        Ok(DecodingState::Finalized) => {
            let result = recognizer.result();
            
            // compensate confidence issues
            if let Some(alternatives) = result.multiple() {
                if let Some(best) = alternatives.alternatives.first() {
                    if !best.text.is_empty() {
                        return Some((best.text.to_string(), best.confidence));
                    }
                }
            }
            
            None
        }
        _ => None,
    }
}


pub fn recognize_speech(data: &[i16]) -> Option<String> {
    let mut recognizer = SPEECH_RECOGNIZER.get()?.lock().unwrap();
    
    match recognizer.accept_waveform(data) {
        Ok(DecodingState::Finalized) => {
            recognizer.result()
                .multiple()
                .and_then(|m| m.alternatives.first().map(|a| a.text.to_string()))
        }
        _ => None,
    }
}


pub fn reset_speech_recognizer() {
    if let Some(recognizer) = SPEECH_RECOGNIZER.get() {
        recognizer.lock().unwrap().reset();
    }
}

pub fn reset_wake_recognizer() {
    if let Some(recognizer) = WAKE_RECOGNIZER.get() {
        recognizer.lock().unwrap().reset();
    }
}

// pub fn recognize(data: &[i16], include_partial: bool) -> Option<String> {
//     let state = RECOGNIZER
//         .get()
//         .unwrap()
//         .lock()
//         .unwrap()
//         .accept_waveform(data);

//     match state {
//         Ok(ds) => {
//             match ds {
//                 DecodingState::Running => {
//                     if include_partial {
//                         Some(
//                             RECOGNIZER
//                                 .get()
//                                 .unwrap()
//                                 .lock()
//                                 .unwrap()
//                                 .partial_result()
//                                 .partial
//                                 .into(),
//                         )
//                     } else {
//                         None
//                     }
//                 }
//                 DecodingState::Finalized => {
//                     // Result will always be multiple because we called set_max_alternatives
//                     RECOGNIZER
//                         .get()
//                         .unwrap()
//                         .lock()
//                         .unwrap()
//                         .result()
//                         .multiple()
//                         .and_then(|m| m.alternatives.first().map(|a| a.text.to_string()))
//                 }
//                 DecodingState::Failed => None,
//             }
//         },
//         Err(err) => {
//                 error!("Vosk accept waveform error.\nError details: {}", err);

//                 None
//         }
//     }
// }

fn get_configured_model_path() -> Result<std::path::PathBuf, String> {
    // try to get from settings
    if let Some(db) = DB.get() {
        let settings = db.read();
        if !settings.vosk_model.is_empty() {
            if let Some(path) = vosk_models::get_model_path(&settings.vosk_model) {
                return Ok(path);
            }
            warn!("Configured Vosk model '{}' not found, falling back to auto-detect", settings.vosk_model);
        }
    }
    
    // auto-detect: use first available model
    let available = vosk_models::scan_vosk_models();
    if let Some(first) = available.first() {
        info!("Auto-detected Vosk model: {}", first.name);
        return Ok(first.path.clone());
    }
    
    // fallback to legacy path
    let legacy_path = std::path::Path::new(config::VOSK_MODEL_PATH);
    if legacy_path.exists() {
        return Ok(legacy_path.to_path_buf());
    }
    
    Err("No Vosk models found".into())
}

// pub fn stereo_to_mono(input_data: &[i16]) -> Vec<i16> {
//     let mut result = Vec::with_capacity(input_data.len() / 2);
//     result.extend(
//         input_data
//             .chunks_exact(2)
//             .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
//     );

//     result
// }
