use discord_rich_presence::activity::Assets;

use crate::discord;
use crate::listen::{ALBUM, ALBUM_ART};

pub fn album(caps: regex::Captures<'_>) {
  let album = caps.get(1).map_or(String::new(), |m| m.as_str().to_string());
  *ALBUM.lock().unwrap() = album;

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
}