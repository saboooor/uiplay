use crate::events::{
  album::album, album_art::album_art, artist::artist, audio_progress::audio_progress,
  connection_request::connection_request, genre::genre, title::title,
};

use regex::Regex;
use std::path::Path;
use std::process::Command;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use tauri::Emitter;

pub static DEVICE_ID: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static DEVICE_NAME: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
static CLIENT_IP: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
static DACP_PORT: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
static ACTIVE_REMOTE: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static TITLE: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static ARTIST: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static ALBUM: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static GENRE: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static ALBUM_ART: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static AUDIO_PROGRESS: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static ALBUM_ART_HASH: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0));

static TITLE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Title: (.*)").unwrap());
static ARTIST_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Artist: (.*)").unwrap());
static ALBUM_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Album: (.*)").unwrap());
static ALBUM_ART_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"coverart size (.*)").unwrap());
static GENRE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Genre: (.*)").unwrap());
static AUDIO_PROGRESS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"audio progress \(min:sec\):\s*(\d+:\d+);\s*remaining:\s*(\d+:\d+);\s*track length\s*(\d+:\d+)").unwrap()
});
static CONNECTION_REQUEST_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"connection request from (.*) with deviceID = (.*)").unwrap());

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
  if ALBUM_ART_REGEX.captures(&message).is_some() {
    let _ = album_art(app.clone()).await;
  }
}

pub fn listen_to_shairport_metadata(
  app: tauri::AppHandle,
  kind: &str,
  code: &str,
  data: Vec<u8>,
  album_art_path: &Path,
) {
  let value = String::from_utf8_lossy(&data).into_owned();
  let message = match (kind, code) {
    ("core", "minm") => Some(format!("Title: {value}")),
    ("core", "asar") => Some(format!("Artist: {value}")),
    ("core", "asal") => Some(format!("Album: {value}")),
    ("core", "asgn") => Some(format!("Genre: {value}")),
    ("ssnc", "cdid") => {
      if let Ok(mut device_id) = DEVICE_ID.lock() {
        *device_id = value.clone();
      }
      shairport_connection_message()
        .or_else(|| Some(format!("Shairport client device ID: {value}")))
    }
    ("ssnc", "clip") | ("ssnc", "conn") => {
      if let Ok(mut client_ip) = CLIENT_IP.lock() {
        *client_ip = value;
      }
      None
    }
    ("ssnc", "dapo") => {
      if let Ok(mut dacp_port) = DACP_PORT.lock() {
        *dacp_port = value;
      }
      None
    }
    ("ssnc", "acre") => {
      if let Ok(mut active_remote) = ACTIVE_REMOTE.lock() {
        *active_remote = value;
      }
      None
    }
    ("ssnc", "snam") => {
      if let Ok(mut device_name) = DEVICE_NAME.lock() {
        *device_name = value;
      }
      shairport_connection_message()
    }
    ("ssnc", "snua") => Some(format!("Client identified as User-Agent: {value}")),
    ("ssnc", "prgr") => shairport_progress_message(&value),
    ("ssnc", "pend") => Some("Connection closed for socket shairport".to_string()),
    ("ssnc", "pbeg") => Some("Accepted Shairport client on socket shairport".to_string()),
    (_, "asfm") => Some(format!("start audio connection, format {value}")),
    ("ssnc", "PICT") => {
      let art_size = data.len();
      if let Err(error) = std::fs::write(album_art_path, data) {
        log_output(app, format!("Failed to save Shairport cover art: {error}"));
        return;
      }
      let album_art_app = app.clone();
      tauri::async_runtime::spawn(async move {
        if let Err(error) = album_art(album_art_app.clone()).await {
          log_output(album_art_app, format!("Failed to process Shairport cover art: {error}"));
        }
      });
      Some(format!("coverart size {art_size}"))
    }
    _ => None,
  };

  if let Some(message) = message {
    let app_for_events = app.clone();
    tauri::async_runtime::spawn(async move {
      listen_to_uxplay_output(app_for_events, message).await;
    });
  }
}

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaybackControl {
  Previous,
  PlayPause,
  Next,
}

#[tauri::command]
pub async fn control_shairport(control: PlaybackControl) -> Result<(), String> {
  let dbus_method = match control {
    PlaybackControl::Previous => "Previous",
    PlaybackControl::PlayPause => "PlayPause",
    PlaybackControl::Next => "Next",
  };
  let mpris_result = send_dbus_control(
    "org.mpris.MediaPlayer2.ShairportSync",
    "/org/mpris/MediaPlayer2",
    &format!("org.mpris.MediaPlayer2.Player.{dbus_method}"),
  );
  if matches!(&mpris_result, Ok(output) if output.status.success()) {
    return Ok(());
  }

  let native_result = send_dbus_control(
    "org.gnome.ShairportSync",
    "/org/gnome/ShairportSync",
    &format!("org.gnome.ShairportSync.RemoteControl.{dbus_method}"),
  );
  if matches!(&native_result, Ok(output) if output.status.success()) {
    return Ok(());
  }

  let client_ip = CLIENT_IP.lock().map_err(|error| error.to_string())?.clone();
  let dacp_port = DACP_PORT.lock().map_err(|error| error.to_string())?.clone();
  let active_remote = ACTIVE_REMOTE.lock().map_err(|error| error.to_string())?.clone();
  if client_ip.is_empty() || dacp_port.is_empty() || active_remote.is_empty() {
    let mpris_error = dbus_error(mpris_result);
    let native_error = dbus_error(native_result);
    return Err(format!(
      "Shairport MPRIS control failed: {mpris_error}. Native control failed: {native_error}. The source also did not provide direct control information"
    ));
  }

  let command = match control {
    PlaybackControl::Previous => "previtem",
    PlaybackControl::PlayPause => "playpause",
    PlaybackControl::Next => "nextitem",
  };
  let host = if client_ip.contains(':') { format!("[{client_ip}]") } else { client_ip };
  // DACP is a LAN-only protocol. Never send its token or request through an
  // HTTP proxy inherited from the desktop environment.
  let client = reqwest::Client::builder()
    .no_proxy()
    .timeout(Duration::from_secs(5))
    .build()
    .map_err(|error| format!("Failed to create the AirPlay control client: {error}"))?;
  let response = client
    .get(format!("http://{host}:{dacp_port}/ctrl-int/1/{command}"))
    .header("Active-Remote", active_remote)
    .header("Connection", "close")
    .send()
    .await
    .map_err(|error| format!("Failed to contact the AirPlay source: {error}"))?;

  if response.status().is_success() {
    Ok(())
  } else {
    Err(format!("AirPlay source rejected the command with status {}", response.status()))
  }
}

fn send_dbus_control(
  destination: &str,
  path: &str,
  method: &str,
) -> std::io::Result<std::process::Output> {
  Command::new("dbus-send")
    .args([
      "--session",
      "--print-reply",
      "--type=method_call",
      &format!("--dest={destination}"),
      path,
      method,
    ])
    .output()
}

fn dbus_error(result: std::io::Result<std::process::Output>) -> String {
  match result {
    Ok(output) => String::from_utf8_lossy(&output.stderr).trim().to_string(),
    Err(error) => error.to_string(),
  }
}

fn shairport_connection_message() -> Option<String> {
  let device_id = DEVICE_ID.lock().ok()?.clone();
  let device_name = DEVICE_NAME.lock().ok()?.clone();
  if device_id.is_empty() || device_name.is_empty() {
    return None;
  }
  Some(format!("connection request from {device_name} with deviceID = {device_id}"))
}

fn shairport_progress_message(value: &str) -> Option<String> {
  let mut timestamps = value.split('/').map(str::parse::<u32>);
  let start = timestamps.next()?.ok()?;
  let current = timestamps.next()?.ok()?;
  let end = timestamps.next()?.ok()?;
  let progress_frames = current.wrapping_sub(start);
  let length_frames = end.wrapping_sub(start);
  if timestamps.next().is_some() || progress_frames > length_frames {
    return None;
  }

  // AirPlay audio timestamps use a 44.1 kHz clock.
  let progress = u64::from(progress_frames) / 44_100;
  let length = u64::from(length_frames) / 44_100;
  let remaining = length.saturating_sub(progress);
  Some(format!(
    "audio progress (min:sec): {}; remaining: {}; track length {}",
    min_sec(progress),
    min_sec(remaining),
    min_sec(length)
  ))
}

fn min_sec(seconds: u64) -> String {
  format!("{}:{:02}", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn converts_shairport_progress_timestamps() {
    assert_eq!(
      shairport_progress_message("1000/2647000/5293000").as_deref(),
      Some("audio progress (min:sec): 1:00; remaining: 1:00; track length 2:00")
    );
  }

  #[test]
  fn converts_wrapped_shairport_progress_timestamps() {
    assert_eq!(
      shairport_progress_message("4294085295/1763999/4410999").as_deref(),
      Some("audio progress (min:sec): 1:00; remaining: 1:00; track length 2:00")
    );
  }
}
