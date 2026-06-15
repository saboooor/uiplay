use crate::discord;
use crate::listen::TITLE;
use tauri::Emitter;


pub fn title(app: tauri::AppHandle, caps: regex::Captures<'_>) {
  let title = caps.get(1).map_or("", |m| m.as_str()).to_string();

  if let Ok(mut cache) = TITLE.lock() {
    if *cache == title {
      return;
    }
    *cache = title.clone();
  }

  // set discord activity
  app.emit("Title", &title).unwrap();
  discord::set_discord_activity();
}
