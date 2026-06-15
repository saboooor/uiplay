mod events;
mod listen;
mod discord;
mod mediaplayer;
mod shairport;
mod uxplay;

use std::fs::create_dir_all;

use tauri::{Manager, path::BaseDirectory};
use tauri_plugin_fs::FsExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .setup(|app| {
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

      // Start the media player process
      if uxplay::is_uxplay_installed() || shairport::is_shairport_installed() {
        listen::log_output(app.handle().clone(), "Starting media streaming process...");
        tauri::async_runtime::spawn(mediaplayer::start_mediaplayer(app.handle().clone()));
      } else {
        listen::log_output(app.handle().clone(), "Neither Shairport-sync nor UxPlay is installed. Please install at least one of them to use UiPlay.");
        return Ok(());
      }

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![mediaplayer::start_mediaplayer])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}