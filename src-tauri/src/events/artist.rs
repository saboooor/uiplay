use crate::discord;
use crate::listen::ARTIST;
use tauri::Emitter;

pub fn artist(app: tauri::AppHandle, caps: regex::Captures<'_>) {
  let artist = caps.get(1).map_or("", |m| m.as_str()).to_string();

  if let Ok(mut cache) = ARTIST.lock() {
    if *cache == artist {
      return;
    }
    *cache = artist.clone();
  }

  // set discord activity
  app.emit("Artist", &artist).unwrap();
  discord::set_discord_activity();
}
