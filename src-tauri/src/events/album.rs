use crate::discord;
use crate::listen::ALBUM;
use tauri::Emitter;

pub fn album(app: tauri::AppHandle, caps: regex::Captures<'_>) {
  let album = caps.get(1).map_or(String::new(), |m| m.as_str().to_string());

  if let Ok(mut cache) = ALBUM.lock() {
    if *cache == album {
      return;
    }
    *cache = album.clone();
  }

  // set discord activity
  app.emit("Album", &album).unwrap();
  discord::set_discord_activity();
}