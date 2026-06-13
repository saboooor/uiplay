use tauri::{Manager, path::BaseDirectory};

#[tauri::command]
pub async fn upload_to_cdn(app: tauri::AppHandle, device_id: String) -> Result<String, String> {
  let art_path = app.path()
    .resolve("uiplay/albumart.png", BaseDirectory::Config)
    .map_err(|e| e.to_string())?;

  // Read the file directly from the backend
  let album_art = std::fs::read(art_path).map_err(|e| e.to_string())?;

  // Use reqwest to upload to your Worker
  let client = reqwest::Client::new();
  let response = client.post("https://uiplay.luminescent.dev/upload")
    .header("Authorization", format!(
      "Bearer {}",
      std::env::var("AUTH_TOKEN").map_err(|e| e.to_string())?
    ))
    .query(&[("device_id", device_id)])
    .body(album_art)
    .send()
    .await
    .map_err(|e| format!("Upload failed: {}", e))?;

  let cdn_url = response.text()
    .await
    .map_err(|e| format!("Failed to read response: {}", e))?;

  Ok(cdn_url)
}