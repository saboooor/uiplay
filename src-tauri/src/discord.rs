use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use regex::Regex;
use std::sync::{Arc, LazyLock, Mutex};

pub struct DiscordState<'a> {
  pub client: DiscordIpcClient,
  pub activity: activity::Activity<'a>,
}

pub type SharedDiscordState<'a> = Arc<Mutex<Option<DiscordState<'a>>>>;
pub static DISCORD_STATE: LazyLock<SharedDiscordState> =
  LazyLock::new(|| Arc::new(Mutex::new(None)));

const ALBUM_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Album: (.*)").unwrap());
const TITLE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Title: (.*)").unwrap());
const ARTIST_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Artist: (.*)").unwrap());
const AUDIO_PROGRESS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"audio progress \(min:sec\): (\d+:\d+); remaining: (\d+:\d+); track length (\d+:\d+)")
    .unwrap()
});

pub async fn process_uxplay_output(output: String) {
  let mut changed = false;

  if let Ok(mut guard) = DISCORD_STATE.lock()
    && let Some(state) = guard.as_mut()
  {
    if let Some(caps) = TITLE_REGEX.captures(&output) {
      let title = caps.get(1).map_or("", |m| m.as_str()).to_string();
      state.activity = state.activity.clone().details(title);
      changed = true;
    }

    if let Some(caps) = ARTIST_REGEX.captures(&output) {
      let artist = caps.get(1).map_or("", |m| m.as_str()).to_string();
      state.activity = state.activity.clone().state(artist);
      changed = true;
    }

    if let Some(caps) = ALBUM_REGEX.captures(&output) {
      let _album = caps.get(1).map_or(String::new(), |m| m.as_str().to_string());
      // Note: Album is parsed but not currently applied to activity fields
    }

    if let Some(caps) = AUDIO_PROGRESS_REGEX.captures(&output) {
      let progress = caps.get(1).map_or("", |m| m.as_str());
      let length = caps.get(3).map_or("", |m| m.as_str());

      fn parse_min_sec(s: &str) -> Option<i64> {
        let mut parts = s.split(':');
        let min = parts.next()?.parse::<i64>().ok()?;
        let sec = parts.next()?.parse::<i64>().ok()?;
        Some(min * 60 + sec)
      }

      let progress_secs = parse_min_sec(progress);
      let length_secs = parse_min_sec(length);

      let now =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
          as i64;

      if let (Some(prog), Some(len)) = (progress_secs, length_secs) {
        let start_ts = now - prog;
        let end_ts = start_ts + len;
        state.activity = state
          .activity
          .clone()
          .timestamps(activity::Timestamps::new().start(start_ts).end(end_ts));
      }
    }

    if changed {
      if let Err(e) = state.client.set_activity(state.activity.clone()) {
        eprintln!("Discord IPC Error: {}", e);
      }
    }
  }
}
