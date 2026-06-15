use crate::discord;
use crate::listen::{ALBUM_ART, ALBUM_ART_HASH, DEVICE_ID, log_output};
use tauri::{Manager, path::BaseDirectory};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{LazyLock, Mutex};
use reqwest::Client;

const APP_SECRET: &str = env!("APP_SECRET");

pub static AUTH_TOKEN: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

pub async fn album_art(app: tauri::AppHandle) -> Result<(), String> {
  let art_path = app
    .path()
    .resolve("uiplay/albumart.png", BaseDirectory::Config)
    .map_err(|e| e.to_string())?;

  // Read the file directly from the backend
  let album_art = std::fs::read(art_path).map_err(|e| e.to_string())?;
  let mut hasher = DefaultHasher::new();
  album_art.hash(&mut hasher);
  let current_hash = hasher.finish();

  if let Ok(cache) = ALBUM_ART_HASH.lock() {
    if *cache == current_hash {
      return Ok(());
    }
  }

  if (*AUTH_TOKEN.lock().unwrap()).is_empty() {
    // Use reqwest to get auth token from worker
    let client = Client::new();
    let response = client
      .post("https://uiplay.luminescent.dev/auth")
      .header("X-App-Secret", APP_SECRET)
      .query(&[("device_id", &*DEVICE_ID.lock().unwrap())])
      .send()
      .await
      .map_err(|e| format!("Auth request failed: {}", e))?;

    let auth_token = response
      .text()
      .await
      .map_err(|e| format!("Failed to read auth response: {}", e))?;

      log_output(app.clone(), format!("Received auth token from worker"));
      *AUTH_TOKEN.lock().unwrap() = auth_token;
  }

  // Use reqwest to upload to worker
  let client = reqwest::Client::new();
  let response = client
    .post("https://uiplay.luminescent.dev/upload")
    .header("Authorization", format!("Bearer {}", AUTH_TOKEN.lock().unwrap()))
    .query(&[("device_id", &*DEVICE_ID.lock().unwrap())])
    .body(album_art)
    .send()
    .await
    .map_err(|e| format!("Upload failed: {}", e))?;

  let cdn_url = response
    .text()
    .await
    .map_err(|e| format!("Failed to read response: {}", e))?;

  if let Ok(mut cache) = ALBUM_ART_HASH.lock() {
    *cache = current_hash;
  }

  // set activity asset
  *ALBUM_ART.lock().unwrap() = cdn_url.clone();
  log_output(app.clone(), format!("Uploaded album art to CDN: {}", cdn_url));

  // set discord activity
  discord::set_discord_activity();

  Ok(())
}
