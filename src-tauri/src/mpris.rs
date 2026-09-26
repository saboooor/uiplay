use crate::listen::{
  ALBUM, ALBUM_ART, ARTIST, AUDIO_PROGRESS, GENRE, PlaybackControl, TITLE, control_playback,
};
use std::collections::HashMap;
use std::sync::{
  OnceLock,
  atomic::{AtomicBool, Ordering},
};
use tauri::AppHandle;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, OwnedValue, Value};

const PATH: &str = "/org/mpris/MediaPlayer2";
static PLAYING: AtomicBool = AtomicBool::new(true);
static CONNECTION: OnceLock<zbus::Connection> = OnceLock::new();

pub fn start(app: AppHandle) {
  tauri::async_runtime::spawn(async move {
    let result = zbus::connection::Builder::session()
      .and_then(|builder| builder.name("org.mpris.MediaPlayer2.UiPlay"))
      .and_then(|builder| builder.serve_at(PATH, MprisRoot))
      .and_then(|builder| builder.serve_at(PATH, MprisPlayer { app }))
      .map_err(|error| error.to_string());

    let Ok(builder) = result else {
      log::warn!("Failed to configure MPRIS service: {}", result.unwrap_err());
      return;
    };
    match builder.build().await {
      Ok(connection) => {
        let _ = CONNECTION.set(connection.clone());
        std::future::pending::<()>().await;
        drop(connection);
      }
      Err(error) => log::warn!("Failed to start MPRIS service: {error}"),
    }
  });
}

struct MprisRoot;

#[zbus::interface(name = "org.mpris.MediaPlayer2")]
impl MprisRoot {
  fn raise(&self) {}
  fn quit(&self) {}

  #[zbus(property)]
  fn can_quit(&self) -> bool {
    false
  }

  #[zbus(property)]
  fn can_raise(&self) -> bool {
    false
  }

  #[zbus(property)]
  fn has_track_list(&self) -> bool {
    false
  }

  #[zbus(property)]
  fn identity(&self) -> &str {
    "UiPlay"
  }

  #[zbus(property)]
  fn desktop_entry(&self) -> &str {
    "uiplay"
  }

  #[zbus(property)]
  fn supported_uri_schemes(&self) -> Vec<String> {
    Vec::new()
  }

  #[zbus(property)]
  fn supported_mime_types(&self) -> Vec<String> {
    Vec::new()
  }
}

struct MprisPlayer {
  app: AppHandle,
}

impl MprisPlayer {
  fn control(&self, control: PlaybackControl) {
    let app = self.app.clone();
    tauri::async_runtime::spawn(async move {
      if let Err(error) = control_playback(app, control).await {
        log::warn!("MPRIS control failed: {error}");
      }
    });
  }

  async fn set_playing(&self, playing: bool, emitter: &SignalEmitter<'_>) {
    if PLAYING.swap(playing, Ordering::Relaxed) != playing {
      self.control(PlaybackControl::PlayPause);
      let _ = self.playback_status_changed(emitter).await;
    }
  }
}

#[zbus::interface(name = "org.mpris.MediaPlayer2.Player")]
impl MprisPlayer {
  fn next(&self) {
    self.control(PlaybackControl::Next);
  }

  fn previous(&self) {
    self.control(PlaybackControl::Previous);
  }

  async fn pause(&self, #[zbus(signal_emitter)] emitter: SignalEmitter<'_>) {
    self.set_playing(false, &emitter).await;
  }

  async fn play_pause(&self, #[zbus(signal_emitter)] emitter: SignalEmitter<'_>) {
    PLAYING.fetch_xor(true, Ordering::Relaxed);
    self.control(PlaybackControl::PlayPause);
    let _ = self.playback_status_changed(&emitter).await;
  }

  async fn stop(&self, #[zbus(signal_emitter)] emitter: SignalEmitter<'_>) {
    self.set_playing(false, &emitter).await;
  }

  async fn play(&self, #[zbus(signal_emitter)] emitter: SignalEmitter<'_>) {
    self.set_playing(true, &emitter).await;
  }

  #[zbus(property)]
  fn playback_status(&self) -> &str {
    if title().is_empty() {
      "Stopped"
    } else if PLAYING.load(Ordering::Relaxed) {
      "Playing"
    } else {
      "Paused"
    }
  }

  #[zbus(property)]
  fn loop_status(&self) -> &str {
    "None"
  }

  #[zbus(property)]
  fn rate(&self) -> f64 {
    1.0
  }

  #[zbus(property)]
  fn shuffle(&self) -> bool {
    false
  }

  #[zbus(property)]
  fn metadata(&self) -> HashMap<String, OwnedValue> {
    metadata()
  }

  #[zbus(property)]
  fn volume(&self) -> f64 {
    1.0
  }

  #[zbus(property)]
  fn position(&self) -> i64 {
    progress_values().0 * 1_000_000
  }

  #[zbus(property)]
  fn minimum_rate(&self) -> f64 {
    1.0
  }

  #[zbus(property)]
  fn maximum_rate(&self) -> f64 {
    1.0
  }

  #[zbus(property)]
  fn can_go_next(&self) -> bool {
    !title().is_empty()
  }

  #[zbus(property)]
  fn can_go_previous(&self) -> bool {
    !title().is_empty()
  }

  #[zbus(property)]
  fn can_play(&self) -> bool {
    !title().is_empty()
  }

  #[zbus(property)]
  fn can_pause(&self) -> bool {
    !title().is_empty()
  }

  #[zbus(property)]
  fn can_seek(&self) -> bool {
    false
  }

  #[zbus(property)]
  fn can_control(&self) -> bool {
    true
  }
}

pub fn metadata_changed() {
  let Some(connection) = CONNECTION.get().cloned() else { return };
  tauri::async_runtime::spawn(async move {
    let Ok(interface) = connection.object_server().interface::<_, MprisPlayer>(PATH).await else {
      return;
    };
    let player = interface.get().await;
    let _ = player.metadata_changed(interface.signal_emitter()).await;
    let _ = player.playback_status_changed(interface.signal_emitter()).await;
    let _ = player.can_go_next_changed(interface.signal_emitter()).await;
    let _ = player.can_go_previous_changed(interface.signal_emitter()).await;
    let _ = player.can_play_changed(interface.signal_emitter()).await;
    let _ = player.can_pause_changed(interface.signal_emitter()).await;
  });
}

fn metadata() -> HashMap<String, OwnedValue> {
  let mut metadata = HashMap::new();
  let track_path = ObjectPath::try_from("/org/mpris/MediaPlayer2/Track/Current").unwrap();
  metadata.insert("mpris:trackid".into(), track_path.into());
  insert_value(&mut metadata, "xesam:title", title());
  insert_value(&mut metadata, "xesam:album", locked(&ALBUM));
  insert_value(&mut metadata, "xesam:artist", vec![locked(&ARTIST)]);
  insert_value(&mut metadata, "xesam:genre", vec![locked(&GENRE)]);
  let length = progress_values().1;
  if length > 0 {
    metadata.insert("mpris:length".into(), (length * 1_000_000).into());
  }
  let art = locked(&ALBUM_ART);
  if !art.is_empty() {
    insert_value(&mut metadata, "mpris:artUrl", art);
  }
  metadata
}

fn insert_value<T>(metadata: &mut HashMap<String, OwnedValue>, key: &str, value: T)
where
  Value<'static>: From<T>,
{
  metadata.insert(key.into(), OwnedValue::try_from(Value::from(value)).unwrap());
}

fn title() -> String {
  locked(&TITLE)
}

fn locked(value: &std::sync::LazyLock<std::sync::Mutex<String>>) -> String {
  value.lock().map(|value| value.clone()).unwrap_or_default()
}

fn progress_values() -> (i64, i64) {
  let value = locked(&AUDIO_PROGRESS);
  let mut values = value.split('|');
  let position = values.next().and_then(parse_time).unwrap_or(0);
  let _remaining = values.next();
  let length = values.next().and_then(parse_time).unwrap_or(0);
  (position, length)
}

fn parse_time(value: &str) -> Option<i64> {
  let (minutes, seconds) = value.split_once(':')?;
  Some(minutes.trim().parse::<i64>().ok()? * 60 + seconds.trim().parse::<i64>().ok()?)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_position() {
    assert_eq!(parse_time("12:34"), Some(754));
  }
}
