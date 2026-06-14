use crate::discord;
use crate::listen::ARTIST;

pub fn artist(caps: regex::Captures<'_>) {
  let artist = caps.get(1).map_or("", |m| m.as_str()).to_string();

  if let Ok(mut cache) = ARTIST.lock() {
    if *cache == artist {
      return;
    }
    *cache = artist.clone();
  }

  // set discord activity
  discord::set_discord_activity();
}
