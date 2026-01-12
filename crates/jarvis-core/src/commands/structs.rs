use std::{collections::HashMap, path::PathBuf, sync::Arc};
use serde::{Serialize, Deserialize};
use parking_lot::RwLock;

#[derive(Serialize, Deserialize, Debug)]
pub struct JCommandsList {
    #[serde(skip)]
    pub path: PathBuf,

    pub commands: Vec<JCommand>,
}



#[derive(Serialize, Deserialize, Debug)]
pub struct JCommand {
    pub id: String,
    pub action: String,
    
    #[serde(default)]
    pub description: String,
    
    #[serde(default)]
    pub exe_path: String,
    
    #[serde(default)]
    pub exe_args: Vec<String>,
    
    #[serde(default)]
    pub cli_cmd: String,
    
    #[serde(default)]
    pub cli_args: Vec<String>,
    
    // #[serde(default)]
    // pub sounds: Vec<String>,

    // Multi-language sounds
    #[serde(default)]
    pub sounds: HashMap<String, Vec<String>>,

    // Multi-language phrases
    #[serde(default)]
    pub phrases: HashMap<String, Vec<String>>,


    // CACHE
    #[serde(skip, default)]
    sounds_cache: RwLock<HashMap<String, Arc<Vec<String>>>>,
    
    #[serde(skip, default)]
    phrases_cache: RwLock<HashMap<String, Arc<Vec<String>>>>,
}

// custom Clone 
impl Clone for JCommand {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            action: self.action.clone(),
            description: self.description.clone(),
            exe_path: self.exe_path.clone(),
            exe_args: self.exe_args.clone(),
            cli_cmd: self.cli_cmd.clone(),
            cli_args: self.cli_args.clone(),
            sounds: self.sounds.clone(),
            phrases: self.phrases.clone(),

            // empty caches for cloned instance
            sounds_cache: RwLock::new(HashMap::new()),
            phrases_cache: RwLock::new(HashMap::new()),
        }
    }
}

impl JCommand {
    // get phrases for current language
    pub fn get_phrases(&self, lang: &str) -> Arc<Vec<String>> {
        if let Some(cached) = self.phrases_cache.read().get(lang) {
            return Arc::clone(cached);
        }
        
        let result = Arc::new(self.resolve_localized(&self.phrases, lang));
        self.phrases_cache.write().insert(lang.to_string(), Arc::clone(&result));
        
        result
    }

    // get all phrases (for backwards compat)
    pub fn get_all_phrases(&self) -> Vec<String> {
        self.phrases.values().flatten().cloned().collect()
    }

    // get sounds for current language
    pub fn get_sounds(&self, lang: &str) -> Arc<Vec<String>> {
        if let Some(cached) = self.sounds_cache.read().get(lang) {
            return Arc::clone(cached);
        }
        
        let result = Arc::new(self.resolve_localized(&self.sounds, lang));
        self.sounds_cache.write().insert(lang.to_string(), Arc::clone(&result));
        
        result
    }

    // get all sounds (for backwards compat)
    pub fn get_all_sounds(&self) -> Vec<String> {
        self.sounds.values().flatten().cloned().collect()
    }


    // shared fallback
    fn resolve_localized(&self, map: &HashMap<String, Vec<String>>, lang: &str) -> Vec<String> {
        // exact match
        if let Some(values) = map.get(lang) {
            return values.clone();
        }

        // fallback to "en"
        if lang != "en" {
            if let Some(values) = map.get("en") {
                return values.clone();
            }
        }

        // fallback to first available
        map.values().next().cloned().unwrap_or_default()
    }
}