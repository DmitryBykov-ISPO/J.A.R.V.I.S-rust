use jarvis_core::voices::{self, structs::VoiceConfig};

#[tauri::command]
pub fn list_voices() -> Vec<VoiceConfig> {
    voices::list_voices()
}

#[tauri::command]
pub fn get_voice(voice_id: String) -> Option<VoiceConfig> {
    voices::get_voice(&voice_id)
}

#[tauri::command]
pub fn preview_voice(voice_id: String) {
    voices::play_preview(&voice_id);
}