use std::{
  io::{BufRead, BufReader},
  process::{Command, Stdio},
  sync::{Arc, Mutex, LazyLock},
  fs::create_dir_all,
};

use tauri::tray::TrayIconBuilder;
use tauri::{path::BaseDirectory, Emitter, Manager};
use tauri_plugin_fs::FsExt;
use regex::Regex;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

struct DiscordState<'a> {
  client: DiscordIpcClient,
  activity: activity::Activity<'a>,
}

type SharedDiscordState<'a> = Arc<Mutex<Option<DiscordState<'a>>>>;
static DISCORD_STATE: LazyLock<SharedDiscordState> = LazyLock::new(|| Arc::new(Mutex::new(None)));

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      let config_dir = app
        .path()
        .resolve("uiplay", BaseDirectory::Config)
        .expect("Failed to resolve config dir");

      create_dir_all(&config_dir).expect("Failed to create config directory");

      let config_dir = config_dir
        .to_string_lossy()
        .to_string();

      // allowed the given directory
      let scope = app.fs_scope();
      let _ = scope.allow_directory(&config_dir, false);

      TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .build(app)?;

      tauri::async_runtime::spawn(start_uxplay(app.handle().clone()));

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![start_uxplay])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
async fn kill_uxplay(app: tauri::AppHandle) {
  let check = Command::new("pgrep").arg("uxplay").output();
  let mut killed = false;
  match check {
    Ok(output) if !output.stdout.is_empty() => {
      log_output(app.clone(), "UxPlay is already running, restarting...");
      // Kill the existing uxplay process and wait for it to exit
      let kill = Command::new("pkill")
        .arg("uxplay")
        .output();
      match kill {
        Ok(_) => {
          killed = true;
          // Wait until no uxplay process is running
          for _ in 0..10 {
            let check_again = Command::new("pgrep")
              .arg("uxplay")
              .output();
            match check_again {
              Ok(out) if out.stdout.is_empty() => break,
              _ => std::thread::sleep(std::time::Duration::from_millis(200)),
            }
          }
        }
        Err(e) => {
          log_output(app.clone(), format!("Failed to kill UxPlay: {}", e));
          return;
        }
      }
    }
    Ok(_) => {
      log_output(app.clone(), "UxPlay is not running, starting a new instance...");
    }
    Err(e) => {
      log_output(app.clone(), format!("Failed to check if UxPlay is running: {}", e));
      return;
    }
  }
  if killed {
    log_output(app.clone(), "UxPlay process killed successfully.");
  }
}

#[tauri::command]
async fn start_uxplay(app: tauri::AppHandle) {
  // First, kill any existing uxplay process
  kill_uxplay(app.clone()).await;

  // Initialize Discord Rich Presence
  let mut client: DiscordIpcClient = DiscordIpcClient::new(
    "1397877327622311997"
  );
  if let Err(e) = client.connect() {
    log_output(app.clone(), format!("Failed to connect to Discord IPC: {:?}", e));
  } else {
    // Create a new activity with the "Listening" type
    let activity: activity::Activity<'_> = activity::Activity::new()
      .activity_type(activity::ActivityType::Listening);

    // Store the client and activity in the shared state
    *DISCORD_STATE.lock().unwrap() = Some(DiscordState {
      client,
      activity,
    });
    log_output(app.clone(), "Connected to Discord IPC and activity set.");
  }

  // import the default GStreamer plugin path
  let default_path = "/usr/lib/gstreamer-1.0";
  let user_path = std::env::var("GST_PLUGIN_PATH").unwrap_or_default();
  let merged = format!("{}:{}", user_path, default_path);

  // Start the uxplay process with stdbuf for line buffering
  let mut child = Command::new("stdbuf")
    // set environment variable for GStreamer plugin path
    .env("GST_PLUGIN_PATH", merged)
    .arg("-oL")
    // set the command to run
    .arg("uxplay")
    // set the app name for uxplay
    .arg("-n")
    .arg("UiPlay")
    // show album art
    .arg("-ca")
    .arg(
      app.path()
        .resolve("uiplay/albumart.png", BaseDirectory::Config)
        .expect("Failed to resolve uiplay/albumart.png")
        .to_string_lossy()
        .to_string(),
    )
    // enable async mode for lossless audio
    .arg("-async")
    // pipe stdout and stderr
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    // spawn the process
    .spawn()
    .expect("Failed to start uxplay with stdbuf");

  // Capture stdout
  let stdout = child.stdout.take().expect("Failed to capture stdout");
  let app_stdout = app.clone();
  std::thread::spawn(move || {
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
      match line {
        Ok(l) => {
          tauri::async_runtime::block_on(process_uxplay_output(l.clone()));
          log_output(app_stdout.clone(), l);
        }
        Err(e) => log_output(app_stdout.clone(), format!("Error reading stdout: {}", e)),
      };
    }
  });

  // Capture stderr
  let stderr = child.stderr.take().expect("Failed to capture stderr");
  let app_stderr = app.clone();
  std::thread::spawn(move || {
    let reader = BufReader::new(stderr);
    for line in reader.lines() {
      match line {
        Ok(l) => log_output(app_stderr.clone(), format!("[STDERR] {}", l)),
        Err(e) => log_output(app_stderr.clone(), format!("Error reading stderr: {}", e)),
      };
    }
  });

  // Wait for the child process to exit
  let status = child.wait().expect("Failed to wait on child");
  
  // Log the exit status
  log_output(app.clone(), format!("UxPlay process exited with status: {}", status));

  // Disconnect from Discord IPC
  let mut discord_state = DISCORD_STATE.lock().unwrap();
  if let Some(state) = discord_state.as_mut() {
    if let Err(e) = state.client.close() {
      log_output(app.clone(), format!("Failed to close Discord IPC: {}", e));
    } else {
      log_output(app.clone(), "Disconnected from Discord IPC successfully.");
    }
    // Remove the state after closing
    *discord_state = None;
  }

  // Attempt to start uxplay again
  log_output(app.clone(), "Trying to start uxplay again...");
  std::thread::spawn(move || {
    tauri::async_runtime::block_on(start_uxplay(app));
  });
}

fn log_output(
  app: tauri::AppHandle,
  output: impl Into<String>,
) {
  let message = output.into();
  println!("{}", message);
  app.emit("uxplay-output", message).unwrap();
}

const ALBUM_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Album: (.*)").unwrap());
const TITLE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Title: (.*)").unwrap());
const ARTIST_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Artist: (.*)").unwrap());
const AUDIO_PROGRESS_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"audio progress \(min:sec\): (\d+:\d+); remaining: (\d+:\d+); track length (\d+:\d+)").unwrap());

async fn process_uxplay_output(
  output: String,
) {
  let mut changed = false;

  if let Ok(mut guard) = DISCORD_STATE.lock() {
    if let Some(state) = guard.as_mut() {

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
        let album = caps.get(1).map_or(String::new(), |m| m.as_str().to_string());
        state.activity = state.activity.clone().assets(
          activity::Assets::new()
            .large_image("albumart")  // replace with album-art-specific URL if needed
            .large_text(album)
            .small_image("icon")
            .small_text("UiPlay"),
        );
        changed = true;
      }

      if let Some(caps) = AUDIO_PROGRESS_REGEX.captures(&output) {
        let progress = caps.get(1).map_or("", |m| m.as_str());
        let length = caps.get(3).map_or("", |m| m.as_str());

        // Helper to convert "min:sec" to seconds
        fn parse_min_sec(s: &str) -> Option<i64> {
          let mut parts = s.split(':');
          let min = parts.next()?.parse::<i64>().ok()?;
          let sec = parts.next()?.parse::<i64>().ok()?;
          Some(min * 60 + sec)
        }

        let progress_secs = parse_min_sec(progress);
        let length_secs = parse_min_sec(length);

        // Use current time as base for start timestamp
        let now = std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap()
          .as_secs() as i64;

        if let (Some(prog), Some(len)) = (progress_secs, length_secs) {
          let start_ts = now - prog;
          let end_ts = start_ts + len;
          state.activity = state.activity.clone().timestamps(
            activity::Timestamps::new()
              .start(start_ts)
              .end(end_ts)
          );
        }
      }

      if changed {
        let _ = state.client.set_activity(state.activity.clone());
      }
    }
  }
  
  return;
}