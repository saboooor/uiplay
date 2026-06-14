use crate::discord;
use crate::listen::{ALBUM, ALBUM_ART, DEVICE_ID};
use discord_rich_presence::activity::Assets;
use tauri::{Manager, path::BaseDirectory};

pub async fn album_art(app: tauri::AppHandle) -> Result<(), String> {
  let art_path = app
    .path()
    .resolve("uiplay/albumart.png", BaseDirectory::Config)
    .map_err(|e| e.to_string())?;

  // Read the file directly from the backend
  let album_art = std::fs::read(art_path).map_err(|e| e.to_string())?;

  let auth_token = std::env::var("AUTH_TOKEN").map_err(|e| e.to_string())?;

  // Use reqwest to upload to your Worker
  let client = reqwest::Client::new();
  let response = client
    .post("https://uiplay.luminescent.dev/upload")
    .header("Authorization", format!("Bearer {}", auth_token))
    .query(&[("device_id", &*DEVICE_ID.lock().unwrap())])
    .body(album_art)
    .send()
    .await
    .map_err(|e| format!("Upload failed: {}", e))?;

  let cdn_url = response
    .text()
    .await
    .map_err(|e| format!("Failed to read response: {}", e))?;

  // set activity asset
  *ALBUM_ART.lock().unwrap() = cdn_url;

  if let Ok(mut guard) = discord::DISCORD_STATE.lock()
    && let Some(state) = guard.as_mut()
  {
    state.activity = state.activity.clone().assets(
      Assets::new()
        .large_image(ALBUM_ART.lock().unwrap().clone())
        .large_text(ALBUM.lock().unwrap().clone())
        .small_image("uiplay")
        .small_text("UiPlay")
    );
  }

  Ok(())
}
