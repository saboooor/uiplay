use crate::{discord::{DISCORD_STATE}, listen::{log_output}};
use discord_rich_presence::{DiscordIpc};
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};

pub fn is_shairport_installed() -> bool {
  std::process::Command::new("which")
    .arg("shairport-sync")
    .output()
    .map(|output| output.status.success())
    .unwrap_or(false)
}

pub async fn kill_shairport(app: tauri::AppHandle) {
  let check = Command::new("pgrep").arg("shairport-sync").output();
  let mut killed = false;
  match check {
    Ok(output) if !output.stdout.is_empty() => {
      log_output(app.clone(), "Shairport is already running, restarting...");
      let kill = Command::new("pkill").arg("shairport-sync").output();
      match kill {
        Ok(_) => {
          killed = true;
          for _ in 0..10 {
            let check_again = Command::new("pgrep").arg("shairport-sync").output();
            match check_again {
              Ok(out) if out.stdout.is_empty() => break,
              _ => std::thread::sleep(std::time::Duration::from_millis(200)),
            }
          }
        }
        Err(e) => {
          log_output(app.clone(), format!("Failed to kill Shairport: {}", e));
          return;
        }
      }
    }
    Ok(_) => {
      log_output(app.clone(), "Shairport is not running, starting a new instance...");
    }
    Err(e) => {
      log_output(app.clone(), format!("Failed to check if Shairport is running: {}", e));
      return;
    }
  }
  if killed {
    log_output(app.clone(), "Shairport process killed successfully.");
  }
}

#[tauri::command]
pub async fn start_shairport(app: tauri::AppHandle) {
  kill_shairport(app.clone()).await;

  let mut child = Command::new("stdbuf")
    .arg("-oL")
    .arg("shairport-sync")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("Failed to start shairport-sync with stdbuf");

  let stdout = child.stdout.take().expect("Failed to capture stdout");
  let app_stdout = app.clone();
  std::thread::spawn(move || {
    let mut reader = BufReader::new(stdout);
    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];

    loop {
      match reader.read(&mut byte) {
        Ok(0) => {
          if !buffer.is_empty() {
            let line = String::from_utf8_lossy(&buffer).to_string();
            log_output(app_stdout.clone(), line);
          }
          break;
        }
        Ok(_) => {
          match byte {
            [b'\n'] | [b'\r'] => {
              if !buffer.is_empty() {
                let line = String::from_utf8_lossy(&buffer).to_string();
                log_output(app_stdout.clone(), line);
                buffer.clear();
              }
            }
            [ch] => buffer.push(ch),
          }
        }
        Err(e) => {
          log_output(app_stdout.clone(), format!("Error reading stdout: {}", e));
          break;
        }
      }
    }
  });

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

  let status = child.wait().expect("Failed to wait on child");
  log_output(app.clone(), format!("Shairport process exited with status: {}", status));

  let mut discord_state = DISCORD_STATE.lock().unwrap();
  if let Some(state) = discord_state.as_mut() {
    if let Err(e) = state.client.close() {
      log_output(app.clone(), format!("Failed to close Discord IPC: {}", e));
    } else {
      log_output(app.clone(), "Disconnected from Discord IPC successfully.");
    }
    *discord_state = None;
  }

  log_output(app.clone(), "Trying to start shairport-sync again...");
  std::thread::spawn(move || {
    tauri::async_runtime::block_on(start_shairport(app));
  });
}
