use crate::{discord, events::{
  album::album, album_art::album_art, artist::artist, audio_progress::audio_progress, connection_request::connection_request, title::title
}};

use regex::Regex;
use std::sync::{LazyLock, Mutex};
use tauri::Emitter;

use discord_rich_presence::DiscordIpc;

pub static DEVICE_ID: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static ALBUM: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static ALBUM_ART: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

const ALBUM_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"Album: (.*)").unwrap());
const ALBUM_ART_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"coverart size (.*)").unwrap()
);
const TITLE_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"Title: (.*)").unwrap()
);
const ARTIST_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"Artist: (.*)").unwrap()
);
const AUDIO_PROGRESS_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"audio progress \(min:sec\): (\d+:\d+); remaining: (\d+:\d+); track length (\d+:\d+)").unwrap()
);
const CONNECTION_REQUEST_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"connection request from (.*) with deviceID = (.*)").unwrap()
);

pub fn log_output(app: tauri::AppHandle, output: impl Into<String>) {
  let message = output.into();

  println!("{}", message);
  app.emit("uxplay-output", &message).unwrap();
}

pub async fn listen_to_uxplay_output(app: tauri::AppHandle, output: String) {
  println!("{}", output);
  app.emit("uxplay-output", &output).unwrap();

  let mut changed = false;
  // caps is the regex captures for each event type, if it matches the output
  if let Some(caps) = CONNECTION_REQUEST_REGEX.captures(&output) {
    connection_request(caps);
    changed = true;
  }
  if let Some(caps) = TITLE_REGEX.captures(&output) {
    title(caps);
    changed = true;
  }
  if let Some(caps) = ARTIST_REGEX.captures(&output) {
    artist(caps);
    changed = true;
  }
  if let Some(caps) = AUDIO_PROGRESS_REGEX.captures(&output) {
    audio_progress(caps);
    changed = true;
  }
  if let Some(caps) = ALBUM_REGEX.captures(&output) {
    album(caps);
    changed = true;
  }
  if let Some(_) = ALBUM_ART_REGEX.captures(&output) {
    let _ = album_art(app.clone()).await;
    changed = true;
  }

  if changed && let Ok(mut guard) = discord::DISCORD_STATE.lock()
    && let Some(state) = guard.as_mut()
  {
    if let Err(e) = state.client.set_activity(state.activity.clone()) {
      log_output(app.clone(), format!("Failed to update Discord activity: {}", e));
    }
  }
}
