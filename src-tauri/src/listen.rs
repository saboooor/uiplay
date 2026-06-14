use crate::events::album_art::album_art;
use crate::events::album::album;
use crate::events::artist::artist;
use crate::events::audio_progress::audio_progress;
use crate::events::title::title;
use crate::events::connection_request::connection_request;

use regex::Regex;
use std::sync::{LazyLock, Mutex};
use tauri::Emitter;

pub static DEVICE_ID: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

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

pub async fn listen_to_uxplay_output(app: tauri::AppHandle, output: impl Into<String>) {
  let message = output.into();

  // caps is the regex captures for each event type, if it matches the output
  if let Some(caps) = CONNECTION_REQUEST_REGEX.captures(&message) {
    connection_request(caps);
  }
  if let Some(_) = ALBUM_ART_REGEX.captures(&message) {
    album_art(app).await.ok();
  }
  if let Some(caps) = TITLE_REGEX.captures(&message) {
    title(caps);
  }
  if let Some(caps) = ARTIST_REGEX.captures(&message) {
    artist(caps);
  }
  if let Some(caps) = ALBUM_REGEX.captures(&message) {
    album(caps);
  }
  if let Some(caps) = AUDIO_PROGRESS_REGEX.captures(&message) {
    audio_progress(caps);
  }
}
