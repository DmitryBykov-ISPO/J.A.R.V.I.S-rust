use jarvis_core::slots;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;

// include core
use jarvis_core::{
    audio, audio_processing, commands, config, db, listener, recorder, stt, intent,
    ipc::{self, IpcAction},
    i18n, voices, models,
    APP_CONFIG_DIR, APP_LOG_DIR, COMMANDS_LIST, DB,
};

// include log
#[macro_use]
extern crate simple_log;
mod log;

// include app
mod app;
mod llm_fallback;

// include tray
// @TODO. macOS currently not supported for tray functionality.
#[cfg(not(target_os = "macos"))]
mod tray;

static SHOULD_STOP: AtomicBool = AtomicBool::new(false);

fn main() -> Result<(), String> {
    eprintln!("[jarvis-app] step: init_dirs");
    config::init_dirs()?;

    eprintln!("[jarvis-app] step: init_logging");
    log::init_logging()?;

    info!("Starting Jarvis v{} ...", config::APP_VERSION.unwrap());
    info!("Config directory is: {}", APP_CONFIG_DIR.get().unwrap().display());
    info!("Log directory is: {}", APP_LOG_DIR.get().unwrap().display());

    eprintln!("[jarvis-app] step: db::init");
    let settings = db::init();

    DB.set(settings.arc().clone())
            .expect("DB already initialized");

    eprintln!("[jarvis-app] step: voices::init");
    let voice_id = settings.lock().voice.clone();
    let language = settings.lock().language.clone();
    if let Err(e) = voices::init(&voice_id, &language) {
        warn!("Failed to init voices: {}", e);
    }

    eprintln!("[jarvis-app] step: i18n::init");
    i18n::init(&settings.lock().language);

    eprintln!("[jarvis-app] step: llm_fallback::init");
    llm_fallback::init();

    eprintln!("[jarvis-app] step: recorder::init");
    if recorder::init().is_err() {
        app::close(1, "recorder::init failed");
    }

    eprintln!("[jarvis-app] step: models::init");
    if let Err(e) = models::init() {
        warn!("Models registry init failed: {}", e);
    }

    eprintln!("[jarvis-app] step: stt::init");
    if stt::init().is_err() {
        app::close(1, "stt::init failed");
    }

    eprintln!("[jarvis-app] step: commands::parse_commands");
    info!("Initializing commands.");
    let cmds = match commands::parse_commands() {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to parse commands: {}. Starting with empty command list.", e);
            Vec::new()
        }
    };
    info!("Commands initialized. Count: {}, List: {:?}", cmds.len(), commands::list_paths(&cmds));
    COMMANDS_LIST.set(cmds).unwrap();

    eprintln!("[jarvis-app] step: audio::init");
    if audio::init().is_err() {
        app::close(1, "audio::init failed");
    }

    eprintln!("[jarvis-app] step: listener::init");
    if let Err(e) = listener::init() {
        error!("Wake-word engine init failed: {}", e);
        app::close(1, "listener::init failed");
    }

    eprintln!("[jarvis-app] step: tokio runtime");
    let rt = Arc::new(
        tokio::runtime::Runtime::new().expect("Failed to create tokio runtime")
    );

    eprintln!("[jarvis-app] step: intent::init");
    rt.block_on(async {
        if let Err(e) = intent::init(COMMANDS_LIST.get().unwrap()).await {
            error!("Failed to initialize intent classifier: {}", e);
            app::close(1, "intent::init failed");
        }
    });

    eprintln!("[jarvis-app] step: slots::init");
    slots::init().map_err(|e| error!("Slot extraction init failed: {}", e)).ok();

    eprintln!("[jarvis-app] step: audio_processing::init");
    info!("Initializing audio processing...");
    if let Err(e) = audio_processing::init() {
        warn!("Audio processing init failed: {}", e);
    }

    eprintln!("[jarvis-app] step: ipc::init");
    info!("Initializing IPC...");
    ipc::init();

    // channel for text commands (manually written in the GUI)
    let (text_cmd_tx, text_cmd_rx) = mpsc::channel::<String>();

    ipc::set_action_handler(move |action| {
        match action {
            IpcAction::Stop => {
                info!("Received stop command from GUI");
                SHOULD_STOP.store(true, Ordering::SeqCst);
            }
            IpcAction::ReloadCommands => {
                info!("Received reload commands request");
                // TODO: implement reload
            }
            IpcAction::SetMuted { muted } => {
                info!("Received mute request: {}", muted);
                // TODO: implement mute
            }
            IpcAction::TextCommand { text } => {
                info!("Received text command: {}", text);
                if let Err(e) = text_cmd_tx.send(text) {
                    error!("Failed to send text command to app: {}", e);
                }
            }
            IpcAction::Ping => {
                // handled internally by server
            }
            _ => {}
        }
    });

    // start WebSocket server on the shared runtime
    let ipc_rt = Arc::clone(&rt);
    std::thread::spawn(move || {
        ipc_rt.block_on(ipc::start_server());
    });
    
    eprintln!("[jarvis-app] step: spawn app thread");
    let app_rt = Arc::clone(&rt);
    std::thread::spawn(move || {
        let _ = app::start(text_cmd_rx, &app_rt);
    });

    eprintln!("[jarvis-app] step: tray::init_blocking");
    tray::init_blocking(settings);

    eprintln!("[jarvis-app] step: main returning Ok");
    Ok(())
}

pub fn should_stop() -> bool {
    SHOULD_STOP.load(Ordering::SeqCst)
}
