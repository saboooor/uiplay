use crate::listen::{listen_to_uxplay_output, log_output};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use tauri::{Manager, path::BaseDirectory};

pub fn is_uxplay_installed() -> bool {
  std::process::Command::new("which")
    .arg("uxplay")
    .output()
    .map(|output| output.status.success())
    .unwrap_or(false)
}

pub async fn kill_uxplay(app: tauri::AppHandle) {
  let check = Command::new("pgrep").arg("uxplay").output();
  let mut killed = false;
  match check {
    Ok(output) if !output.stdout.is_empty() => {
      log_output(app.clone(), "UxPlay is already running, restarting...");
      let kill = Command::new("pkill").arg("uxplay").output();
      match kill {
        Ok(_) => {
          killed = true;
          for _ in 0..10 {
            let check_again = Command::new("pgrep").arg("uxplay").output();
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

pub async fn start_uxplay(app: tauri::AppHandle, name: String) {
  let mut command = Command::new("stdbuf");
  command
    .arg("-oL")
    .arg("uxplay")
    .arg("-n")
    .arg(name)
    .arg("-ca")
    .arg(
      app
        .path()
        .resolve("uiplay/albumart.png", BaseDirectory::Config)
        .expect("Failed to resolve uiplay/albumart.png")
        .to_string_lossy()
        .to_string(),
    )
    .arg("-async")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

  // AppImages prepend bundled library and GStreamer paths to the environment.
  // Those files belong to UiPlay, not to the system-installed UxPlay. Passing
  // them on makes UxPlay ignore or reject its matching system plugins.
  sanitize_appimage_environment(&mut command);

  // Do not override GST_PLUGIN_PATH. GStreamer already knows the correct system
  // plugin directory, including the distribution and CPU architecture in use.
  let mut child = match command.spawn() {
    Ok(child) => child,
    Err(error) => {
      log_output(app, format!("Failed to start UxPlay: {}", error));
      return;
    }
  };

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
            tauri::async_runtime::block_on(listen_to_uxplay_output(app_stdout.clone(), line));
          }
          break;
        }
        Ok(_) => match byte {
          [b'\n'] | [b'\r'] => {
            if !buffer.is_empty() {
              let line = String::from_utf8_lossy(&buffer).to_string();
              tauri::async_runtime::block_on(listen_to_uxplay_output(app_stdout.clone(), line));
              buffer.clear();
            }
          }
          [ch] => buffer.push(ch),
        },
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

  let status = match child.wait() {
    Ok(status) => status,
    Err(error) => {
      log_output(app.clone(), format!("Failed to wait for UxPlay: {}", error));
      return;
    }
  };
  log_output(app.clone(), format!("UxPlay process exited with status: {}", status));

  if !status.success() {
    log_output(app, "UxPlay stopped after an error. Fix the error and restart it from UiPlay.");
  }
}

fn sanitize_appimage_environment(command: &mut Command) {
  let Some(app_dir) = std::env::var_os("APPDIR") else {
    return;
  };
  let app_dir = Path::new(&app_dir);

  for variable in [
    "PATH",
    "LD_LIBRARY_PATH",
    "GST_PLUGIN_PATH",
    "GST_PLUGIN_PATH_1_0",
    "GST_PLUGIN_SYSTEM_PATH",
    "GST_PLUGIN_SYSTEM_PATH_1_0",
  ] {
    remove_appimage_paths(command, variable, app_dir);
  }

  for variable in ["GST_PLUGIN_SCANNER", "GST_PLUGIN_SCANNER_1_0"] {
    if std::env::var_os(variable).is_some_and(|path| Path::new(&path).starts_with(app_dir)) {
      command.env_remove(variable);
    }
  }
}

fn remove_appimage_paths(command: &mut Command, variable: &str, app_dir: &Path) {
  let Some(paths) = std::env::var_os(variable) else {
    return;
  };
  let host_paths: Vec<_> = std::env::split_paths(&paths)
    .filter(|path| !path.as_os_str().is_empty() && !path.starts_with(app_dir))
    .collect();

  command.env_remove(variable);
  if !host_paths.is_empty()
    && let Ok(joined_paths) = std::env::join_paths(host_paths)
  {
    command.env(variable, joined_paths);
  }
}
