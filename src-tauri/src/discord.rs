use discord_rich_presence::{DiscordIpcClient, activity::{self, Assets}};
use std::sync::{Arc, LazyLock, Mutex};
use discord_rich_presence::DiscordIpc;

use crate::listen::{ALBUM, ALBUM_ART, ARTIST, TITLE};

pub struct DiscordState {
  pub client: DiscordIpcClient,
  pub activity: activity::Activity<'static>,
}

pub type SharedDiscordState = Arc<Mutex<Option<DiscordState>>>;
pub static DISCORD_STATE: LazyLock<SharedDiscordState> =
  LazyLock::new(|| Arc::new(Mutex::new(None)));

pub fn set_discord_activity() {
  let arc = DISCORD_STATE.clone();
  if let Ok(mut guard) = arc.lock() && let Some(state) = guard.as_mut() {
    state.activity = state.activity.clone()
      .details(TITLE.lock().unwrap().clone())
      .state(ARTIST.lock().unwrap().clone())
      .assets(
        Assets::new()
          .large_image(ALBUM_ART.lock().unwrap().clone())
          .large_text(ALBUM.lock().unwrap().clone())
          .small_image("uiplay")
          .small_text("UiPlay")
      );

    if let Err(e) = state.client.set_activity(state.activity.clone()) {
      eprintln!("Failed to update Discord activity: {}", e);
    }
  }
}