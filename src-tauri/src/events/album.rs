use crate::discord::ACTIVITY_ASSETS;

pub fn album(caps: regex::Captures<'_>) {
  let album = caps.get(1).map_or(String::new(), |m| m.as_str().to_string());
  *ACTIVITY_ASSETS.lock().unwrap() = ACTIVITY_ASSETS.lock().unwrap().clone()
    .large_text(album);
}