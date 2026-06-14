use crate::discord;
use discord_rich_presence::{DiscordIpc, activity};

use tauri::{Manager, path::BaseDirectory};

#[tauri::command]
pub async fn upload_to_cdn(app: tauri::AppHandle, device_id: String) -> Result<String, String> {
  let art_path =
    app.path().resolve("uiplay/albumart.png", BaseDirectory::Config).map_err(|e| e.to_string())?;

  // Read the file directly from the backend
  let album_art = std::fs::read(art_path).map_err(|e| e.to_string())?;

  // Use reqwest to upload to your Worker
  let client = reqwest::Client::new();
  let response = client
    .post("https://uiplay.luminescent.dev/upload")
    .header(
      "Authorization",
      format!("Bearer {}", std::env::var("AUTH_TOKEN").map_err(|e| e.to_string())?),
    )
    .query(&[("device_id", device_id)])
    .body(album_art)
    .send()
    .await
    .map_err(|e| format!("Upload failed: {}", e))?;

  let cdn_url = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

  // set discord activity with the CDN URL
  if let Ok(mut guard) = discord::DISCORD_STATE.lock()
    && let Some(state) = guard.as_mut()
  {
    state.activity =
      state.activity.clone().assets(activity::Assets::new().large_image(cdn_url.clone()));
    if let Err(e) = state.client.set_activity(state.activity.clone()) {
      crate::log_output(app.clone(), format!("Failed to update Discord activity: {}", e));
    }
  }

  Ok(cdn_url)
}
