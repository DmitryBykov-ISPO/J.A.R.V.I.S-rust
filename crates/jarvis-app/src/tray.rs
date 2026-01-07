mod menu;

use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};
use winit::event_loop::{ControlFlow, EventLoopBuilder};
use image;
use std::process::Command;

#[cfg(target_os="windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;

use jarvis_core::{config, i18n, ipc::{self, IpcEvent}};

const TRAY_ICON_BYTES: &[u8] = include_bytes!("../../../resources/icons/32x32.png");

pub fn init_blocking() {
    // load tray icon
    //let icon_path = format!("{}/../../resources/icons/{}", env!("CARGO_MANIFEST_DIR"), config::TRAY_ICON);
    //let icon = load_icon(std::path::Path::new(&icon_path));
    let icon = load_icon_from_bytes(TRAY_ICON_BYTES);

    // form tray menu
    // let tray_menu = Menu::with_items(&[
    //     &MenuItem::new("Перезапуск", true, None),
    //     &MenuItem::new("Настройки", true, None),
    //     &MenuItem::new("Выход", true, None),
    // ])
    // .unwrap();

    let tray_menu = Menu::with_items(&[
        &MenuItem::with_id("restart", i18n::t("tray-restart"), true, None),
        &MenuItem::with_id("settings", i18n::t("tray-settings"), true, None),
        &MenuItem::with_id("exit", i18n::t("tray-exit"), true, None),
    ]).unwrap();

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip(i18n::t("tray-tooltip"))
        .with_icon(icon)
        .build()
        .unwrap();

    let menu_channel = MenuEvent::receiver();
    // let tray_channel = TrayIconEvent::receiver();

    // @TODO: Test on Linux
    // We need gtk for the tray icon to show up, we need to initialize gtk and create the tray_icon
    #[cfg(target_os = "linux")]
    {
        gtk::init().unwrap();
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok(event) = menu_channel.try_recv() {
                handle_menu_event(&event);
            }
            glib::ControlFlow::Continue
        });
        gtk::main();
    }

    // @TODO: Test on MacOS
    #[cfg(target_os = "macos")]
    {
        // macOS needs proper run loop - tao or winit on main thread
        use winit::event_loop::{EventLoop, ControlFlow};
        let event_loop = EventLoop::new().unwrap();
        event_loop.run(move |_event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);
            if let Ok(event) = menu_channel.try_recv() {
                handle_menu_event(&event);
            }
        }).unwrap();
    }

    #[cfg(target_os = "windows")]
    {
        // simple polling works on Windows
        loop {
            if let Ok(event) = menu_channel.try_recv() {
                handle_menu_event(&event);
            }
            
            // pump Windows messages
            unsafe {
                let mut msg: winapi::um::winuser::MSG = std::mem::zeroed();
                while winapi::um::winuser::PeekMessageW(
                    &mut msg, 
                    std::ptr::null_mut(), 
                    0, 0, 
                    winapi::um::winuser::PM_REMOVE
                ) != 0 {
                    winapi::um::winuser::TranslateMessage(&msg);
                    winapi::um::winuser::DispatchMessageW(&msg);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    info!("Tray initialized.");
}

fn handle_menu_event(event: &MenuEvent) {
    match event.id.0.as_str() {
        "exit" => std::process::exit(0),
        "restart" => {
            info!("Restarting from tray menu...");
            restart_app();
        }
        "settings" => {
            info!("Opening settings from tray menu...");
            open_settings();
        }
        _ => {}
    }
}

fn load_icon_from_bytes(bytes: &[u8]) -> tray_icon::Icon {
    let image = image::load_from_memory(bytes)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    tray_icon::Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

fn restart_app() {
    // get current executable path
    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to get executable path: {}", e);
            return;
        }
    };
    
    // spawn new instance
    match Command::new(&exe_path).spawn() {
        Ok(_) => {
            info!("Spawned new instance, exiting current...");
            std::process::exit(0);
        }
        Err(e) => {
            error!("Failed to restart: {}", e);
        }
    }
}

fn open_settings() {
    // check if jarvis-gui is connected via IPC
    if ipc::has_clients() {
        // gui is running, send reveal event
        info!("GUI is connected, sending reveal event");
        ipc::send(IpcEvent::RevealWindow);
    } else {
        // gui not running, launch it
        info!("GUI not connected, launching jarvis-gui");
        launch_gui();
    }
}

fn launch_gui() {
    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to get executable path: {}", e);
            return;
        }
    };
    
    // jarvis-gui should be in same directory as jarvis-app
    let gui_path = exe_path.parent()
        .map(|p| p.join(get_gui_executable_name()))
        .unwrap_or_else(|| get_gui_executable_name().into());
    
    info!("Launching GUI: {:?}", gui_path);
    
    match Command::new(&gui_path).spawn() {
        Ok(_) => {
            info!("Launched jarvis-gui");
        }
        Err(e) => {
            error!("Failed to launch jarvis-gui: {}", e);
        }
    }
}

#[cfg(target_os = "windows")]
fn get_gui_executable_name() -> &'static str {
    "jarvis-gui.exe"
}

#[cfg(not(target_os = "windows"))]
fn get_gui_executable_name() -> &'static str {
    "jarvis-gui"
}