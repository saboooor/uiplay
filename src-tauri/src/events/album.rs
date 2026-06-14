use crate::discord;
use crate::listen::{ALBUM};

pub fn album(caps: regex::Captures<'_>) {
  let album = caps.get(1).map_or(String::new(), |m| m.as_str().to_string());

  if let Ok(mut cache) = ALBUM.lock() {
    if *cache == album {
      return;
    }
    *cache = album;
  }

  // set discord activity
  discord::set_discord_activity();
}