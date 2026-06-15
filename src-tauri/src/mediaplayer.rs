use crate::{discord::{DISCORD_STATE, DiscordState}, listen::{log_output}, uxplay, shairport};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};

#[tauri::command]
pub async fn start_mediaplayer(app: tauri::AppHandle) {
  // Initialize Discord
  let mut client = DiscordIpcClient::new("1397877327622311997");
  if let Err(e) = client.connect() {
    log_output(app.clone(), format!("Failed to connect to Discord IPC: {:?}", e));
  } else {
    let activity = activity::Activity::new()
      .activity_type(activity::ActivityType::Listening);

    *DISCORD_STATE.lock().unwrap() = Some(DiscordState { client, activity });
    log_output(app.clone(), "Connected to Discord IPC and activity set.");
  }

  // Start media player
  if uxplay::is_uxplay_installed() {
    uxplay::start_uxplay(app).await;
    return;
  }
  if shairport::is_shairport_installed() {
    shairport::start_shairport(app).await;
    return;
  }
}
