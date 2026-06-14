use crate::discord;

pub fn artist(caps: regex::Captures<'_>) {
  // set discord activity
  if let Ok(mut guard) = discord::DISCORD_STATE.lock()
    && let Some(state) = guard.as_mut()
  {
    let artist = caps.get(1).map_or("", |m| m.as_str()).to_string();
    state.activity = state.activity.clone().state(artist);
  }
}
