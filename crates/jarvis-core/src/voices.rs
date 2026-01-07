use std::fs;
use std::path::{Path, PathBuf};
use rand::prelude::*;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
// use chrono::Timelike;

use crate::{DB, SOUND_DIR, audio, config, time};

pub mod structs;

static VOICES: OnceCell<Vec<structs::VoiceConfig>> = OnceCell::new();
static CURRENT_VOICE_ID: OnceCell<RwLock<String>> = OnceCell::new();

pub fn init(default_voice: &str) -> Result<(), String> {
    CURRENT_VOICE_ID.get_or_init(|| RwLock::new(default_voice.to_string()));
    
    let voices = scan_voices()?;
    
    if voices.is_empty() {
        return Err("No voices found".into());
    }
    
    info!("Loaded {} voice(s): {:?}", 
        voices.len(), 
        voices.iter().map(|v| &v.voice.id).collect::<Vec<_>>()
    );
    
    VOICES.set(voices).map_err(|_| "Voices already initialized")?;
    
    Ok(())
}

pub fn scan_voices() -> Result<Vec<structs::VoiceConfig>, String> {
    let voices_dir = SOUND_DIR.join(&config::VOICES_PATH);
    
    if !voices_dir.exists() {
        return Err(format!("Voices directory not found: {:?}", voices_dir));
    }
    
    let mut voices = Vec::new();
    
    let entries = fs::read_dir(&voices_dir)
        .map_err(|e| format!("Failed to read voices directory: {}", e))?;
    
    for entry in entries.flatten() {
        let voice_path = entry.path();
        if !voice_path.is_dir() {
            continue;
        }
        
        let toml_path = voice_path.join("voice.toml");
        if !toml_path.exists() {
            warn!("Voice folder {:?} missing voice.toml, skipping", voice_path);
            continue;
        }
        
        match load_voice_config(&toml_path, &voice_path) {
            Ok(config) => voices.push(config),
            Err(e) => warn!("Failed to load voice {:?}: {}", voice_path, e),
        }
    }
    
    Ok(voices)
}

fn load_voice_config(toml_path: &Path, voice_path: &Path) -> Result<structs::VoiceConfig, String> {
    let content = fs::read_to_string(toml_path)
        .map_err(|e| format!("Failed to read voice.toml: {}", e))?;
    
    let mut config: structs::VoiceConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse voice.toml: {}", e))?;
    
    config.path = voice_path.to_path_buf();
    
    Ok(config)
}



pub fn list_voices() -> Vec<structs::VoiceConfig> {
    VOICES.get().cloned().unwrap_or_default()
}

pub fn get_voice(voice_id: &str) -> Option<structs::VoiceConfig> {
    VOICES.get()?.iter().find(|v| v.voice.id == voice_id).cloned()
}

pub fn get_current_voice() -> Option<structs::VoiceConfig> {
    let current_id = CURRENT_VOICE_ID.get()?.read().clone();
    get_voice(&current_id)
}

pub fn set_current_voice(voice_id: &str) {
    if let Some(lock) = CURRENT_VOICE_ID.get() {
        *lock.write() = voice_id.to_string();
    }
}

fn get_current_language() -> String {
    DB.get()
        .map(|db| db.read().language.clone())
        .unwrap_or_else(|| "ru".to_string())
}



fn find_sound_file(voice_path: &Path, lang: &str, sound_name: &str) -> Option<PathBuf> {
    let extensions = ["mp3", "wav", "ogg"];
    let lang_path = voice_path.join(lang);
    
    // try language subfolder first
    for ext in &extensions {
        let file_path = lang_path.join(format!("{}.{}", sound_name, ext));
        if file_path.exists() {
            return Some(file_path);
        }
    }
    
    // fallback to root voice folder
    for ext in &extensions {
        let file_path = voice_path.join(format!("{}.{}", sound_name, ext));
        if file_path.exists() {
            return Some(file_path);
        }
    }
    
    None
}

fn play_random_from(sounds: &[String]) {
    if sounds.is_empty() {
        return;
    }
    
    let voice = match get_current_voice() {
        Some(v) => v,
        None => {
            warn!("No current voice set");
            return;
        }
    };
    
    let lang = get_current_language();
    let sound_name = sounds.choose(&mut rand::thread_rng()).unwrap();
    
    match find_sound_file(&voice.path, &lang, sound_name) {
        Some(path) => {
            debug!("Playing: {:?}", path);
            audio::play_sound(&path);
        }
        None => {
            warn!("Sound not found: {} (lang: {}, voice: {})", sound_name, lang, voice.voice.id);
        }
    }
}

pub fn play(reaction: structs::Reaction) {
    let voice = match get_current_voice() {
        Some(v) => v,
        None => {
            warn!("No current voice set");
            return;
        }
    };
    
    let sounds = match reaction {
        structs::Reaction::Greet => {
            // try time specific first
            let time_specific = match time::TimeOfDay::now() {
                time::TimeOfDay::Morning => &voice.reactions.greet_morning,
                time::TimeOfDay::Day => &voice.reactions.greet_day,
                time::TimeOfDay::Evening => &voice.reactions.greet_evening,
                time::TimeOfDay::Night => &voice.reactions.greet_night,
            };

            if time_specific.is_empty() {
                // fallback to simple run voice (not time specific)
                &voice.reactions.greet
            } else {
                time_specific
            }
        }
        structs::Reaction::Reply => &voice.reactions.reply,
        structs::Reaction::Ok => &voice.reactions.ok,
        structs::Reaction::NotFound => &voice.reactions.not_found,
        structs::Reaction::Thanks => &voice.reactions.thanks,
        structs::Reaction::Error => &voice.reactions.error,
        structs::Reaction::Goodbye => &voice.reactions.goodbye,
    };
    
    play_random_from(sounds);
}

// Play a preview sound for a specific voice
pub fn play_preview(voice_id: &str) {
    let voice = match get_voice(voice_id) {
        Some(v) => v,
        None => {
            warn!("Voice not found for preview: {}", voice_id);
            return;
        }
    };
    
    let lang = get_current_language();
    
    // pick from reply or ok sounds for preview
    let sounds: Vec<&String> = voice.reactions.reply.iter()
        .chain(voice.reactions.ok.iter())
        .chain(voice.reactions.greet.iter())
        .collect();
    
    if sounds.is_empty() {
        warn!("No preview sounds for voice: {}", voice_id);
        return;
    }
    
    let sound_name = sounds.choose(&mut rand::thread_rng()).unwrap();
    
    if let Some(path) = find_sound_file(&voice.path, &lang, sound_name) {
        debug!("Playing preview: {:?}", path);
        audio::play_sound(&path);
    }
}


// shortcuts
pub fn play_greet() { play(structs::Reaction::Greet); } // app startup
pub fn play_reply() { play(structs::Reaction::Reply); } // wake word detected
pub fn play_ok() { play(structs::Reaction::Ok); } // command executed
pub fn play_not_found() { play(structs::Reaction::NotFound); }
pub fn play_thanks() { play(structs::Reaction::Thanks); }
pub fn play_error() { play(structs::Reaction::Error); }
pub fn play_goodbye() { play(structs::Reaction::Goodbye); }