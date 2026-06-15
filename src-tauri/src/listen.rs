use crate::{events::{
  album::album, album_art::album_art, artist::artist, genre::genre, audio_progress::audio_progress, connection_request::connection_request, title::title
}};

use regex::Regex;
use std::sync::{LazyLock, Mutex};
use tauri::Emitter;

pub static DEVICE_ID: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static TITLE: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static ARTIST: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static ALBUM: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static GENRE: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static ALBUM_ART: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static AUDIO_PROGRESS: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static ALBUM_ART_HASH: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0));

const TITLE_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"Title: (.*)").unwrap());
const ARTIST_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"Artist: (.*)").unwrap());
const ALBUM_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"Album: (.*)").unwrap());
const ALBUM_ART_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"coverart size (.*)").unwrap());
const GENRE_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"Genre: (.*)").unwrap());
const AUDIO_PROGRESS_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"audio progress \(min:sec\):\s*(\d+:\d+);\s*remaining:\s*(\d+:\d+);\s*track length\s*(\d+:\d+)").unwrap());
const CONNECTION_REQUEST_REGEX: LazyLock<Regex> = LazyLock::new(||
  Regex::new(r"connection request from (.*) with deviceID = (.*)").unwrap());

pub fn log_output(app: tauri::AppHandle, output: impl Into<String>) {
  let message = output.into();

  println!("{}", message);
  app.emit("app-output", &message).unwrap();
}

pub async fn listen_to_uxplay_output(app: tauri::AppHandle, output: impl Into<String>) {
  let message = output.into();

  println!("{}", message);
  app.emit("uxplay-output", &message).unwrap();

  // caps is the regex captures for each event type, if it matches the output
  if let Some(caps) = TITLE_REGEX.captures(&message) {
    title(app.clone(), caps);
  }
  if let Some(caps) = ARTIST_REGEX.captures(&message) {
    artist(app.clone(), caps);
  }
  if let Some(caps) = ALBUM_REGEX.captures(&message) {
    album(app.clone(), caps);
  }
  if let Some(caps) = GENRE_REGEX.captures(&message) {
    genre(app.clone(), caps);
  }
  if let Some(caps) = AUDIO_PROGRESS_REGEX.captures(&message) {
    audio_progress(caps);
  }
  if let Some(caps) = CONNECTION_REQUEST_REGEX.captures(&message) {
    connection_request(caps);
  }
  if let Some(_) = ALBUM_ART_REGEX.captures(&message) {
    let _ = album_art(app.clone()).await;
  }
}
