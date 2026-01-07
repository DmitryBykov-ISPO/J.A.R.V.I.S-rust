// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use jarvis_core::{config, db, i18n, APP_CONFIG_DIR, APP_LOG_DIR, DB};

use parking_lot::RwLock;
use std::sync::Arc;

#[macro_use]
extern crate simple_log;

mod events;

mod tauri_commands;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<RwLock<db::structs::Settings>>,
}

fn main() {
    config::init_dirs().expect("Failed to init dirs");
    
    // basic logging setup (simpler for GUI)
    simple_log::quick!("info");

    // init db
    let settings = db::init_settings();

    // init i18n
    i18n::init(&settings.language);

    // set db
    DB.set(Arc::new(RwLock::new(settings)))
            .expect("DB already initialized");
    let db_arc = DB.get().unwrap().clone();

    tauri::Builder::default()
        .manage(AppState { db: db_arc })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // audio
            tauri_commands::pv_get_audio_devices,
            tauri_commands::pv_get_audio_device_name,
            tauri_commands::play_sound,

            // db
            tauri_commands::db_read,
            tauri_commands::db_write,

            // etc
            tauri_commands::get_app_version,
            tauri_commands::get_author_name,
            tauri_commands::get_repository_link,
            tauri_commands::get_tg_official_link,
            tauri_commands::get_feedback_link,

            // fs
            tauri_commands::get_log_file_path,
            tauri_commands::show_in_folder,

            // sys
            tauri_commands::get_current_ram_usage,
            tauri_commands::get_peak_ram_usage,
            tauri_commands::get_cpu_temp,
            tauri_commands::get_cpu_usage,
            tauri_commands::get_jarvis_app_stats,
            tauri_commands::is_jarvis_app_running,
            tauri_commands::run_jarvis_app,

            // vosk
            tauri_commands::list_vosk_models,

            // i18n
            tauri_commands::get_translations,
            tauri_commands::translate,
            tauri_commands::get_current_language,
            tauri_commands::set_language,
            tauri_commands::get_supported_languages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}