use crate::discord;
use crate::listen::AUDIO_PROGRESS;
use discord_rich_presence::{activity::Timestamps};

pub fn audio_progress(caps: regex::Captures<'_>) {
  let progress = caps.get(1).map_or("", |m| m.as_str());
  let remaining = caps.get(2).map_or("", |m| m.as_str());
  let length = caps.get(3).map_or("", |m| m.as_str());
  let cache_key = format!("{}|{}|{}", progress, remaining, length);

  if let Ok(mut cache) = AUDIO_PROGRESS.lock() {
    if *cache == cache_key {
      return;
    }
    *cache = cache_key;
  }

  if let Ok(mut guard) = discord::DISCORD_STATE.lock()
    && let Some(state) = guard.as_mut()
  {
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
        state.activity = state.activity.clone()
          .timestamps(
            Timestamps::new()
              .start(start_ts)
              .end(end_ts)
          );
      }
  }
}
