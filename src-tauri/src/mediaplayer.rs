use crate::{
  discord::{DISCORD_STATE, DiscordState},
  listen::log_output,
  settings::{self, Provider},
  shairport, uxplay,
};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};

#[tauri::command]
pub async fn start_mediaplayer(app: tauri::AppHandle) {
  let settings = settings::load(&app);

  // Initialize Discord
  let mut client = DiscordIpcClient::new("1397877327622311997");
  if let Err(e) = client.connect() {
    log_output(app.clone(), format!("Failed to connect to Discord IPC: {:?}", e));
  } else {
    let activity = activity::Activity::new().activity_type(activity::ActivityType::Listening);

    *DISCORD_STATE.lock().unwrap() = Some(DiscordState { client, activity });
    log_output(app.clone(), "Connected to Discord IPC and activity set.");
  }

  // Stop either receiver left by a previous selection before starting the
  // preferred provider. If it is unavailable, fall back to the other one.
  uxplay::kill_uxplay(app.clone()).await;
  shairport::kill_shairport(app.clone()).await;

  match settings.provider {
    Provider::Shairport if shairport::is_shairport_installed() => {
      shairport::start_shairport(app, settings.name).await;
    }
    Provider::Uxplay if uxplay::is_uxplay_installed() => {
      uxplay::start_uxplay(app, settings.name).await;
    }
    Provider::Shairport if uxplay::is_uxplay_installed() => {
      log_output(app.clone(), "Shairport Sync is unavailable; falling back to UxPlay.");
      uxplay::start_uxplay(app, settings.name).await;
    }
    Provider::Uxplay if shairport::is_shairport_installed() => {
      log_output(app.clone(), "UxPlay is unavailable; falling back to Shairport Sync.");
      shairport::start_shairport(app, settings.name).await;
    }
    _ => log_output(app, "Neither Shairport Sync nor UxPlay is installed."),
  }
}
