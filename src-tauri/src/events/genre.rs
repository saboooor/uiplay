use crate::listen::GENRE;
use tauri::Emitter;

pub fn genre(app: tauri::AppHandle, caps: regex::Captures<'_>) {
  let genre = caps.get(1).map_or("", |m| m.as_str()).to_string();

  if let Ok(mut cache) = GENRE.lock() {
    if *cache == genre {
      return;
    }
    *cache = genre.clone();
  }

  app.emit("Genre", &genre).unwrap();
}
