use crate::events::{
  album::album, album_art::album_art, artist::artist, audio_progress::audio_progress,
  connection_request::connection_request, genre::genre, title::title,
};

use regex::Regex;
use std::path::Path;
use std::process::Command;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use tauri::{Emitter, Manager, path::BaseDirectory};

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
static UXPLAY_DACP_ENDPOINT: LazyLock<Mutex<Option<CachedDacpEndpoint>>> =
  LazyLock::new(|| Mutex::new(None));

pub fn reset_session_state(app: &tauri::AppHandle) {
  for value in [
    &DEVICE_ID,
    &DEVICE_NAME,
    &CLIENT_IP,
    &DACP_PORT,
    &ACTIVE_REMOTE,
    &TITLE,
    &ARTIST,
    &ALBUM,
    &GENRE,
    &ALBUM_ART,
    &AUDIO_PROGRESS,
  ] {
    if let Ok(mut value) = value.lock() {
      value.clear();
    }
  }
  if let Ok(mut hash) = ALBUM_ART_HASH.lock() {
    *hash = 0;
  }
  if let Ok(mut endpoint) = UXPLAY_DACP_ENDPOINT.lock() {
    *endpoint = None;
  }
  for event in ["Title", "Artist", "Album", "Genre"] {
    let _ = app.emit(event, "");
  }
  crate::mpris::metadata_changed();
}

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
  let _ = app.emit("app-output", &message);
}

pub async fn listen_to_uxplay_output(app: tauri::AppHandle, output: impl Into<String>) {
  let message = output.into();

  println!("{}", message);
  let _ = app.emit("uxplay-output", &message);

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
pub async fn control_shairport(
  app: tauri::AppHandle,
  control: PlaybackControl,
) -> Result<(), String> {
  control_playback(app, control).await
}

pub async fn control_playback(
  app: tauri::AppHandle,
  control: PlaybackControl,
) -> Result<(), String> {
  let command = match control {
    PlaybackControl::Previous => "previtem",
    PlaybackControl::PlayPause => "playpause",
    PlaybackControl::Next => "nextitem",
  };

  // Both receivers ultimately use DACP. Shairport supplies the endpoint in
  // metadata; UxPlay exports DACP-ID and Active-Remote to a structured file.
  let dacp_error = match dacp_endpoint(&app).await {
    Ok((host, port, active_remote)) => {
      return send_dacp_command(&host, port, &active_remote, command).await;
    }
    Err(error) => error,
  };

  // Older Shairport configurations may expose controls only through D-Bus.
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

  let mpris_error = dbus_error(mpris_result);
  let native_error = dbus_error(native_result);
  Err(format!(
    "DACP control is unavailable: {dacp_error}. Shairport MPRIS control failed: {mpris_error}. Native control failed: {native_error}"
  ))
}

async fn send_dacp_command(
  host: &str,
  port: u16,
  active_remote: &str,
  command: &str,
) -> Result<(), String> {
  let url_host = if host.contains(':') { format!("[{host}]") } else { host.to_string() };
  // DACP is a LAN-only protocol. Never send its token or request through an
  // HTTP proxy inherited from the desktop environment.
  let client = reqwest::Client::builder()
    .no_proxy()
    .timeout(Duration::from_secs(5))
    .build()
    .map_err(|error| format!("Failed to create the AirPlay control client: {error}"))?;
  let response = client
    .get(format!("http://{url_host}:{port}/ctrl-int/1/{command}"))
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

async fn dacp_endpoint(app: &tauri::AppHandle) -> Result<(String, u16, String), String> {
  let client_ip = CLIENT_IP.lock().map_err(|error| error.to_string())?.clone();
  let port = DACP_PORT.lock().map_err(|error| error.to_string())?.parse::<u16>().ok();
  let shairport_remote = ACTIVE_REMOTE.lock().map_err(|error| error.to_string())?.clone();
  if !client_ip.is_empty()
    && let Some(port) = port
    && !shairport_remote.is_empty()
  {
    return Ok((client_ip, port, shairport_remote));
  }

  let path = app
    .path()
    .resolve("uiplay/uxplay.dacp", BaseDirectory::Config)
    .map_err(|error| error.to_string())?;
  let contents = std::fs::read_to_string(path)
    .map_err(|error| format!("UxPlay has not exported DACP credentials: {error}"))?;
  let credentials = parse_uxplay_dacp(&contents)?;
  if let Ok(cache) = UXPLAY_DACP_ENDPOINT.lock()
    && let Some(endpoint) = cache.as_ref()
    && endpoint.dacp_id == credentials.dacp_id
    && endpoint.active_remote == credentials.active_remote
  {
    return Ok((endpoint.host.clone(), endpoint.port, endpoint.active_remote.clone()));
  }
  let service_name = format!("iTunes_Ctrl_{}", credentials.dacp_id);
  let endpoint = tauri::async_runtime::spawn_blocking(move || resolve_dacp_service(&service_name))
    .await
    .map_err(|error| format!("DACP resolver task failed: {error}"))??;
  if let Ok(mut cache) = UXPLAY_DACP_ENDPOINT.lock() {
    *cache = Some(CachedDacpEndpoint {
      dacp_id: credentials.dacp_id,
      active_remote: credentials.active_remote.clone(),
      host: endpoint.0.clone(),
      port: endpoint.1,
    });
  }
  Ok((endpoint.0, endpoint.1, credentials.active_remote))
}

struct CachedDacpEndpoint {
  dacp_id: String,
  active_remote: String,
  host: String,
  port: u16,
}

struct DacpCredentials {
  dacp_id: String,
  active_remote: String,
}

fn parse_uxplay_dacp(contents: &str) -> Result<DacpCredentials, String> {
  let mut dacp_id = None;
  let mut active_remote = None;
  let lines: Vec<_> = contents.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
  for line in &lines {
    if let Some((key, value)) = line.split_once([':', '=']) {
      match key.trim().to_ascii_lowercase().as_str() {
        "dacp-id" => dacp_id = Some(value.trim().to_string()),
        "active-remote" => active_remote = Some(value.trim().to_string()),
        _ => {}
      }
    }
  }
  // UxPlay 1.73 exports two bare lines: DACP-ID followed by Active-Remote.
  if dacp_id.is_none() && active_remote.is_none() && lines.len() >= 2 {
    dacp_id = Some(lines[0].to_string());
    active_remote = Some(lines[1].to_string());
  }
  match (dacp_id, active_remote) {
    (Some(dacp_id), Some(active_remote)) if !dacp_id.is_empty() && !active_remote.is_empty() => {
      Ok(DacpCredentials { dacp_id, active_remote })
    }
    _ => Err("UxPlay's DACP export is incomplete".into()),
  }
}

fn resolve_dacp_service(service_name: &str) -> Result<(String, u16), String> {
  let output = Command::new("avahi-browse")
    .args(["--resolve", "--cache", "--parsable", "--no-db-lookup", "_dacp._tcp"])
    .output()
    .map_err(|error| format!("Failed to run avahi-browse: {error}"))?;
  if !output.status.success() {
    return Err(format!("avahi-browse failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
  }
  parse_avahi_dacp_output(&String::from_utf8_lossy(&output.stdout), service_name)
}

fn parse_avahi_dacp_output(output: &str, service_name: &str) -> Result<(String, u16), String> {
  for line in output.lines() {
    let fields: Vec<_> = line.split(';').collect();
    if fields.first() == Some(&"=")
      && fields.get(3) == Some(&service_name)
      && fields.get(4) == Some(&"_dacp._tcp")
      && let (Some(address), Some(port)) = (fields.get(7), fields.get(8))
      && let Ok(port) = port.parse()
    {
      return Ok((unescape_avahi_field(address), port));
    }
  }
  Err(format!("DACP service {service_name} was not found"))
}

fn unescape_avahi_field(value: &str) -> String {
  let bytes = value.as_bytes();
  let mut result = String::with_capacity(value.len());
  let mut index = 0;
  while index < bytes.len() {
    if bytes[index] == b'\\'
      && index + 3 < bytes.len()
      && bytes[index + 1..index + 4].iter().all(u8::is_ascii_digit)
    {
      let number =
        (bytes[index + 1] - b'0') * 100 + (bytes[index + 2] - b'0') * 10 + bytes[index + 3] - b'0';
      result.push(number as char);
      index += 4;
    } else {
      result.push(bytes[index] as char);
      index += 1;
    }
  }
  result
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

  #[test]
  fn parses_uxplay_dacp_export() {
    let credentials = parse_uxplay_dacp("A1B2C3D4\n123456789\n").unwrap();
    assert_eq!(credentials.dacp_id, "A1B2C3D4");
    assert_eq!(credentials.active_remote, "123456789");
  }

  #[test]
  fn parses_labeled_uxplay_dacp_export() {
    let credentials = parse_uxplay_dacp("DACP-ID: A1B2C3D4\nActive-Remote: 123456789\n").unwrap();
    assert_eq!(credentials.dacp_id, "A1B2C3D4");
    assert_eq!(credentials.active_remote, "123456789");
  }

  #[test]
  fn parses_uxplay_dacp_export_with_equals() {
    let credentials = parse_uxplay_dacp("DACP-ID=A1B2C3D4\nActive-Remote=123456789\n").unwrap();
    assert_eq!(credentials.dacp_id, "A1B2C3D4");
    assert_eq!(credentials.active_remote, "123456789");
  }

  #[test]
  fn rejects_incomplete_uxplay_dacp_export() {
    assert!(parse_uxplay_dacp("DACP-ID: A1B2C3D4\n").is_err());
  }

  #[test]
  fn unescapes_avahi_fields() {
    assert_eq!(unescape_avahi_field("iTunes\\032Control"), "iTunes Control");
  }

  #[test]
  fn parses_avahi_dacp_endpoint() {
    let output =
      "=;wlan0;IPv6;iTunes_Ctrl_A1B2C3D4;_dacp._tcp;local;phone.local;192.168.2.99;51197;\n";
    assert_eq!(
      parse_avahi_dacp_output(output, "iTunes_Ctrl_A1B2C3D4").unwrap(),
      ("192.168.2.99".into(), 51197)
    );
  }
}
