use crate::listen::ALBUM;

pub fn album(caps: regex::Captures<'_>) {
  let album = caps.get(1).map_or(String::new(), |m| m.as_str().to_string());
  *ALBUM.lock().unwrap() = album;
}