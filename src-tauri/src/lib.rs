mod events;
mod listen;
mod discord;
mod uxplay;

use std::fs::create_dir_all;

use tauri::tray::TrayIconBuilder;
use tauri::{Manager, path::BaseDirectory};
use tauri_plugin_fs::FsExt;

use crate::listen::{log_output};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .setup(|app| {
      // Check if uxplay is installed before proceeding
      if !uxplay::is_uxplay_installed() {
        log_output(
          app.handle().clone(),
          "UxPlay is not installed."
        );
        return Ok(());
      }

      // Initialize logging
      if cfg!(debug_assertions) {
        app
          .handle()
          .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())?;
      }

      // Resolve the config directory for UiPlay and create it if it doesn't exist
      let config_dir =
        app.path().resolve("uiplay", BaseDirectory::Config).expect("Failed to resolve config dir");
      create_dir_all(&config_dir).expect("Failed to create config directory");

      // Convert the config directory path to a string for fs scope
      let config_dir = config_dir.to_string_lossy().to_string();

      // allowed the given directory
      let scope = app.fs_scope();
      let _ = scope.allow_directory(&config_dir, false);

      // Create a system tray icon
      TrayIconBuilder::new().icon(app.default_window_icon().unwrap().clone()).build(app)?;

      // Start the uxplay process
      tauri::async_runtime::spawn(uxplay::start_uxplay(app.handle().clone()));

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![uxplay::start_uxplay])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}