mod intentclassifier;

use std::path::PathBuf;

use crate::{JCommandsList, commands::JCommand, config};
use once_cell::sync::OnceCell;
use crate::config::structs::IntentRecognitionEngine;

static IRE_TYPE: OnceCell<IntentRecognitionEngine> = OnceCell::new();

pub async fn init(commands: &Vec<JCommandsList>) -> Result<(), String> {
    if IRE_TYPE.get().is_some() {
        return Ok(());
    } // already initialized

    // set default ire type
    // @TODO. Make it configurable?
    IRE_TYPE.set(config::DEFAULT_INTENT_RECOGNITION_ENGINE).unwrap();

    // load given recorder
    match IRE_TYPE.get().unwrap() {
        IntentRecognitionEngine::IntentClassifier => {
            info!("Initializing IRE backend.");
            intentclassifier::init(&commands).await?;
            info!("IRE backend initialized.");
        },
        IntentRecognitionEngine::Rasa => todo!(),
    }

    Ok(())
}

pub async fn classify(text: &str) -> Option<(String, f64)> {
    match IRE_TYPE.get()? {
        IntentRecognitionEngine::IntentClassifier => {
            match intentclassifier::classify(text).await {
                Ok(prediction) => {
                    let confidence = prediction.confidence.value();
                    if confidence >= config::INTENT_CLASSIFIER_MIN_CONFIDENCE {
                        Some((prediction.intent.to_string(), confidence))
                    } else {
                        None
                    }
                }
                Err(e) => {
                    error!("Intent classification error: {}", e);
                    None
                }
            }
        }
        IntentRecognitionEngine::Rasa => todo!(),
    }
}

pub fn get_command_by_intent(commands: &'static Vec<JCommandsList>, intent_id: &str) -> Option<(&'static PathBuf, &'static JCommand)> {
    match IRE_TYPE.get()? {
        IntentRecognitionEngine::IntentClassifier => {
            intentclassifier::get_command(commands, intent_id)
        }
        IntentRecognitionEngine::Rasa => todo!(),
    }
}