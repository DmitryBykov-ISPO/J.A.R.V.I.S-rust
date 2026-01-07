use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// include core
use jarvis_core::{
    audio, audio_processing, commands, config, db, listener, recorder, stt, intent,
    ipc::{self, IpcAction},
    i18n,
    APP_CONFIG_DIR, APP_LOG_DIR, COMMANDS_LIST, DB,
};

// include log
#[macro_use]
extern crate simple_log;
mod log;

// include app
mod app;

// include tray
// @TODO. macOS currently not supported for tray functionality.
#[cfg(not(target_os = "macos"))]
mod tray;

static SHOULD_STOP: AtomicBool = AtomicBool::new(false);

fn main() -> Result<(), String> {
    // initialize directories
    config::init_dirs()?;

    // initialize logging
    log::init_logging()?;

    // log some base info
    info!("Starting Jarvis v{} ...", config::APP_VERSION.unwrap());
    info!("Config directory is: {}", APP_CONFIG_DIR.get().unwrap().display());
    info!("Log directory is: {}", APP_LOG_DIR.get().unwrap().display());

    // initialize database (settings)
    DB.set(Arc::new(RwLock::new(db::init_settings())))
            .expect("DB already initialized");

    // init i18n
    i18n::init(&DB.get().unwrap().read().language);

    // initialize tray
    // @TODO. macOS currently not supported for tray functionality,
    // due to the separate thread in which tray processing works,
    // but macOS requires it to be processed in the main thread only
    // The solution may be to include wake-word detection etc. in the winit event loop. (only for MacOS, though?)
    //#[cfg(not(target_os = "macos"))]
    //tray::init();

    // init recorder
    if recorder::init().is_err() {
        app::close(1);
    }

    // init stt engine
    if stt::init().is_err() {
        // @TODO. Allow continuing even without STT, if commands is using keywords or smthng?
        app::close(1); // cannot continue without stt
    }

    // init tts engine
    // none for now (Silero-rs coming)

    // init commands
    info!("Initializing commands.");
    let cmds = match commands::parse_commands() {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to parse commands: {}. Starting with empty command list.", e);
            Vec::new()
        }
    };
    info!("Commands initialized. Count: {}, List: {:?}", cmds.len(), commands::list(&cmds));
    COMMANDS_LIST.set(cmds).unwrap();

    // init audio
    if audio::init().is_err() {
        // @TODO. Allow continuing even without audio?
        app::close(1); // cannot continue without audio
    }

    // init wake-word engine
    if listener::init().is_err() {
        app::close(1);  // cannot continue without wake-word engine
    }

    // init intent-recognition engine
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        if intent::init(COMMANDS_LIST.get().unwrap()).await.is_err() {
            error!("Failed to initialize intent classifier");
            app::close(1);
        }
    });

    // init audio processing
    info!("Initializing audio processing...");
    if let Err(e) = audio_processing::init() {
        warn!("Audio processing init failed: {}", e);
    }

    // init IPC
    info!("Initializing IPC...");
    ipc::init();

    ipc::set_action_handler(|action| {
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
            _ => {}
        }
    });

    // start WebSocket server for ipc
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime for IPC");
        rt.block_on(ipc::start_server());
    });
    
    // start the app (in the background thread)
    std::thread::spawn(|| {
        let _ = app::start();
    });

    tray::init_blocking();

    Ok(())
}

pub fn should_stop() -> bool {
    SHOULD_STOP.load(Ordering::SeqCst)
}