use jarvis_core::{db, DB};
use crate::AppState;

#[tauri::command]
pub fn db_read(state: tauri::State<'_, AppState>, key: &str) -> String {
    let settings = state.db.read();

    match key {
        "selected_microphone" => settings.microphone.to_string(),
        "assistant_voice" => settings.voice.clone(),
        "selected_wake_word_engine" => format!("{:?}", settings.wake_word_engine),
        "selected_intent_recognition_engine" => format!("{:?}", settings.intent_recognition_engine),
        "speech_to_text_engine" => format!("{:?}", settings.speech_to_text_engine),
        "api_key__picovoice" => settings.api_keys.picovoice.clone(),
        "api_key__openai" => settings.api_keys.openai.clone(),
        _ => String::new(),
    }
}

#[tauri::command]
pub fn db_write(state: tauri::State<'_, AppState>, key: &str, val: &str) -> bool {
    let snapshot = {
        let mut settings = state.db.write();

        match key {
            "selected_microphone" => {
                if let Ok(v) = val.parse::<i32>() {
                    // info!("MICROPHONE changed: {}", v);
                    settings.microphone = v;
                } else {
                    return false;
                }
            }
            "assistant_voice" => {
                settings.voice = val.to_string();
            }
            "selected_wake_word_engine" => {
                match val.to_lowercase().as_str() {
                    "rustpotter" => settings.wake_word_engine = jarvis_core::config::structs::WakeWordEngine::Rustpotter,
                    "vosk" => settings.wake_word_engine = jarvis_core::config::structs::WakeWordEngine::Vosk,
                    "porcupine" => settings.wake_word_engine = jarvis_core::config::structs::WakeWordEngine::Porcupine,
                    _ => return false,
                }
            }
            "selected_intent_recognition_engine" => {
                match val.to_lowercase().as_str() {
                    "intentclassifier" => settings.intent_recognition_engine = jarvis_core::config::structs::IntentRecognitionEngine::IntentClassifier,
                    "rasa" => settings.intent_recognition_engine = jarvis_core::config::structs::IntentRecognitionEngine::Rasa,
                    _ => return false,
                }
            }
            "api_key__picovoice" => {
                settings.api_keys.picovoice = val.to_string();
            }
            "api_key__openai" => {
                settings.api_keys.openai = val.to_string();
            }
            _ => return false,
        }

        settings.clone()
    };

    // save to disk
    if let Err(e) = db::save_settings(&snapshot) {
        info!("SETTINGS NOT SAVED");
    }

    true
}
