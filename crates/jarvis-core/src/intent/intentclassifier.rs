use intent_classifier::{
    IntentClassifier, IntentPrediction, IntentError,
    TrainingExample, TrainingSource, IntentId
};

use tokio::sync::OnceCell;
use std::path::PathBuf;
use std::fs;

use crate::commands::{self, JCommand, JCommandsList};
use crate::{APP_CONFIG_DIR, i18n};

static CLASSIFIER: OnceCell<IntentClassifier> = OnceCell::const_new();
// static COMMANDS_MAP: OnceCell<Vec<JCommandsList>> = OnceCell::const_new();

const TRAINING_CACHE_FILE: &str = "intent_training.json";
const COMMANDS_HASH_FILE: &str = "commands_hash.txt";

pub async fn init(commands: &[JCommandsList]) -> Result<(), String> {
    // parse commands first
    // let commands = commands::parse_commands()?;
    let current_hash = commands::commands_hash(&commands); // regen hash for current commands set
    
    // init classifier
    let classifier = IntentClassifier::new().await
        .map_err(|e| format!("Failed to init IntentClassifier: {}", e))?;
    
    // check if we can use cached training data
    let config_dir = APP_CONFIG_DIR.get().ok_or("Config dir not set")?;
    let hash_path = config_dir.join(COMMANDS_HASH_FILE);
    let cache_path = config_dir.join(TRAINING_CACHE_FILE);
    
    let should_retrain = if hash_path.exists() && cache_path.exists() {
        let stored_hash = fs::read_to_string(&hash_path).unwrap_or_default();
        stored_hash.trim() != current_hash
    } else {
        true
    };
    
    if should_retrain {
        info!("Training intent classifier with {} commands...", commands.len());
        train_classifier(&classifier, &commands).await?;
        
        // save training data and hash
        if let Ok(export) = classifier.export_training_data().await {
            let _ = fs::write(&cache_path, export);
            let _ = fs::write(&hash_path, &current_hash);
            info!("Training data cached.");
        }
    } else {
        info!("Loading cached training data...");
        if let Ok(data) = fs::read_to_string(&cache_path) {
            classifier.import_training_data(&data).await
                .map_err(|e| format!("Failed to import training data: {}", e))?;
        }
    }
    
    // store data
    CLASSIFIER.set(classifier).map_err(|_| "Classifier already set")?;
    // COMMANDS_MAP.set(commands).map_err(|_| "Commands map already set")?;
    
    Ok(())
}

pub async fn classify(text: &str) -> Result<IntentPrediction, IntentError> {
    let classifier = CLASSIFIER.get().expect("IntentClassifier not initialized");
    classifier.predict_intent(text).await
}

// get command by intent ID
pub fn get_command(commands: &'static [JCommandsList], intent_id: &str) -> Option<(&'static PathBuf, &'static JCommand)> {
    // let commands = COMMANDS_MAP.get()?;
    
    for assistant_cmd in commands {
        for cmd in &assistant_cmd.commands {
            if cmd.id == intent_id {
                return Some((&assistant_cmd.path, cmd));
            }
        }
    }
    
    None
}

// based on: https://github.com/ciresnave/intent-classifier/blob/main/examples/basic_usage.rs
async fn train_classifier(
    classifier: &IntentClassifier,
    commands: &[JCommandsList]
) -> Result<(), String> {
    let lang = i18n::get_language();
    info!("Training intent classifier for language: {}", lang);

    let mut total_examples = 0;

    for assistant_cmd in commands {
        for cmd in &assistant_cmd.commands {
            // use language-specific phrases
            let phrases = cmd.get_phrases(&lang);
            
            for phrase in phrases.iter() {
                let example = TrainingExample {
                    text: phrase.clone(),
                    intent: IntentId::from(cmd.id.as_str()),
                    confidence: 1.0,
                    source: TrainingSource::Programmatic,
                };
                
                classifier.add_training_example(example).await
                    .map_err(|e| format!("Failed to add training example: {}", e))?;
                
                total_examples += 1;
            }
        }
    }

    info!("Added {} training examples for language '{}'", total_examples, lang);
    Ok(())
}