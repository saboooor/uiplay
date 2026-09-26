use crate::{
  discord::{DISCORD_STATE, DiscordState},
  listen::{log_output, reset_session_state},
  settings::{self, Provider},
  shairport::{self, ShairportProcess},
  uxplay,
};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use std::process::Child;
use std::sync::Mutex;
use std::time::{Duration, Instant};

enum ReceiverProcess {
  Uxplay(Child),
  Shairport(ShairportProcess),
}

impl ReceiverProcess {
  fn stop(&mut self) -> Result<(), String> {
    let child = match self {
      Self::Uxplay(child) => child,
      Self::Shairport(process) => &mut process.child,
    };
    if child.try_wait().map_err(|error| error.to_string())?.is_none() {
      // Give the receiver a chance to release its sockets and mDNS records.
      // Escalate only if it ignores SIGTERM.
      let terminated = unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) } == 0;
      if terminated {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
          if child.try_wait().map_err(|error| error.to_string())?.is_some() {
            break;
          }
          std::thread::sleep(Duration::from_millis(50));
        }
      }
      if child.try_wait().map_err(|error| error.to_string())?.is_none() {
        child.kill().map_err(|error| error.to_string())?;
      }
    }
    child.wait().map_err(|error| error.to_string())?;
    if let Self::Shairport(process) = self {
      process.stop_metadata();
    }
    Ok(())
  }
}

#[derive(Default)]
pub struct ReceiverManager {
  process: Mutex<Option<ReceiverProcess>>,
}

impl ReceiverManager {
  fn restart(&self, app: tauri::AppHandle) -> Result<(), String> {
    let mut process = self.process.lock().map_err(|error| error.to_string())?;
    if let Some(mut running) = process.take() {
      log_output(app.clone(), "Stopping the current receiver...");
      if let Err(error) = running.stop() {
        log_output(app.clone(), format!("Failed to stop the receiver cleanly: {error}"));
      }
    }
    reset_session_state(&app);

    let settings = settings::load(&app);
    let (provider, running) = match settings.provider {
      Provider::Shairport if shairport::is_shairport_installed() => (
        "Shairport Sync",
        ReceiverProcess::Shairport(shairport::start_shairport(app.clone(), settings.name)?),
      ),
      Provider::Uxplay if uxplay::is_uxplay_installed() => {
        ("UxPlay", ReceiverProcess::Uxplay(uxplay::start_uxplay(app.clone(), settings.name)?))
      }
      Provider::Shairport if uxplay::is_uxplay_installed() => {
        log_output(app.clone(), "Shairport Sync is unavailable; falling back to UxPlay.");
        ("UxPlay", ReceiverProcess::Uxplay(uxplay::start_uxplay(app.clone(), settings.name)?))
      }
      Provider::Uxplay if shairport::is_shairport_installed() => {
        log_output(app.clone(), "UxPlay is unavailable; falling back to Shairport Sync.");
        (
          "Shairport Sync",
          ReceiverProcess::Shairport(shairport::start_shairport(app.clone(), settings.name)?),
        )
      }
      _ => return Err("Neither Shairport Sync nor UxPlay is installed.".into()),
    };

    log_output(app, format!("{provider} receiver started."));
    *process = Some(running);
    Ok(())
  }
}

impl Drop for ReceiverManager {
  fn drop(&mut self) {
    if let Ok(process) = self.process.get_mut()
      && let Some(mut process) = process.take()
    {
      let _ = process.stop();
    }
  }
}

fn connect_discord(app: &tauri::AppHandle) {
  let Ok(mut state) = DISCORD_STATE.lock() else { return };
  if state.is_some() {
    return;
  }
  let mut client = DiscordIpcClient::new("1397877327622311997");
  if let Err(error) = client.connect() {
    log_output(app.clone(), format!("Failed to connect to Discord IPC: {error:?}"));
  } else {
    let activity = activity::Activity::new().activity_type(activity::ActivityType::Listening);
    *state = Some(DiscordState { client, activity });
    log_output(app.clone(), "Connected to Discord IPC.");
  }
}

#[tauri::command]
pub fn start_mediaplayer(
  app: tauri::AppHandle,
  manager: tauri::State<'_, ReceiverManager>,
) -> Result<(), String> {
  connect_discord(&app);
  manager.restart(app)
}
