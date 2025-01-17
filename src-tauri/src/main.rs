#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::process;

use nix::libc::geteuid;
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};

mod events_service;

mod logging;

#[tokio::main]
async fn main() {
    // Check if the current user is root
    if unsafe { geteuid() == 0 } {
        println!("Running with root privileges.");
    } else {
        eprintln!("This program needs to be run as root. Please use sudo.");
        process::exit(1); // Exit if not root
    }
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            let tray_id = "my-tray";
            tokio::spawn(events_service::start_emitting_events(app_handle.clone()));
            logging::start_log_monitoring(app_handle.clone());
            SystemTray::new()
                .with_id(tray_id)
                .with_menu(
                    SystemTrayMenu::new()
                        .add_item(CustomMenuItem::new("quit", "Quit"))
                        .add_item(CustomMenuItem::new("open", "Open")),
                )
                .on_event(move |_event| {
                    let _tray_handle = app_handle.tray_handle_by_id(tray_id).unwrap();
                })
                .build(app)?;
            Ok(())
        })
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "open" => {
                    let main_window = app.get_window("main").unwrap();
                    main_window.show().unwrap();
                    main_window.set_focus().unwrap();
                }
                _ => {}
            },
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
