use crate::listen::{listen_to_shairport_metadata, log_output};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};
use tauri::{Manager, path::BaseDirectory};

pub fn is_shairport_installed() -> bool {
  Command::new("shairport-sync")
    .arg("-V")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .is_ok_and(|status| status.success())
}

pub struct ShairportProcess {
  pub child: Child,
  metadata_stop: Arc<AtomicBool>,
  metadata_pipe: PathBuf,
  metadata_reader: Option<std::thread::JoinHandle<()>>,
}

impl ShairportProcess {
  pub fn stop_metadata(&mut self) {
    self.metadata_stop.store(true, Ordering::Relaxed);
    if let Ok(mut pipe) = OpenOptions::new().write(true).open(&self.metadata_pipe) {
      let _ = pipe.write_all(b"\n");
    }
    if let Some(reader) = self.metadata_reader.take() {
      let _ = reader.join();
    }
  }
}

pub fn start_shairport(app: tauri::AppHandle, name: String) -> Result<ShairportProcess, String> {
  let config_dir = match app.path().resolve("uiplay", BaseDirectory::Config) {
    Ok(path) => path,
    Err(error) => {
      log_output(app, format!("Failed to resolve the UiPlay config directory: {error}"));
      return Err(format!("Failed to resolve the UiPlay config directory: {error}"));
    }
  };
  let metadata_pipe = config_dir.join("shairport-metadata");
  let album_art = config_dir.join("albumart.png");
  let shairport_config = config_dir.join("shairport-sync.conf");
  if let Err(error) = fs::write(
    &shairport_config,
    r#"general = {
  dbus_service_bus = "session";
  mpris_service_bus = "session";
};
"#,
  ) {
    log_output(app, format!("Failed to write the Shairport Sync configuration: {error}"));
    return Err(format!("Failed to write the Shairport Sync configuration: {error}"));
  }
  if let Err(error) = create_metadata_pipe(&metadata_pipe) {
    log_output(app, format!("Failed to create the Shairport metadata pipe: {error}"));
    return Err(format!("Failed to create the Shairport metadata pipe: {error}"));
  }

  // Opening both ends keeps startup deterministic: neither this reader nor
  // Shairport Sync has to wait for the other process to open the FIFO first.
  let metadata = match OpenOptions::new().read(true).write(true).open(&metadata_pipe) {
    Ok(file) => file,
    Err(error) => {
      log_output(app, format!("Failed to open the Shairport metadata pipe: {error}"));
      return Err(format!("Failed to open the Shairport metadata pipe: {error}"));
    }
  };

  let mut child = match Command::new("shairport-sync")
    .arg("-c")
    .arg(shairport_config)
    .arg("-a")
    .arg(name)
    .arg("-M")
    .arg(format!("--metadata-pipename={}", metadata_pipe.display()))
    .arg("-g")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
  {
    Ok(child) => child,
    Err(error) => {
      return Err(format!("Failed to start Shairport Sync: {error}"));
    }
  };

  let metadata_stop = Arc::new(AtomicBool::new(false));
  let metadata_reader =
    spawn_metadata_reader(app.clone(), metadata, album_art, metadata_stop.clone());
  if let Some(stdout) = child.stdout.take() {
    spawn_log_reader(app.clone(), stdout, "[SHAIRPORT]");
  }
  if let Some(stderr) = child.stderr.take() {
    spawn_log_reader(app.clone(), stderr, "[SHAIRPORT STDERR]");
  }

  Ok(ShairportProcess {
    child,
    metadata_stop,
    metadata_pipe,
    metadata_reader: Some(metadata_reader),
  })
}

fn create_metadata_pipe(path: &Path) -> std::io::Result<()> {
  match fs::symlink_metadata(path) {
    Ok(metadata) if metadata.file_type().is_fifo() => return Ok(()),
    Ok(_) => fs::remove_file(path)?,
    Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
    Err(_) => {}
  }

  let path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, "pipe path contains a NUL byte")
  })?;
  // SAFETY: path is a valid, NUL-terminated pathname and mode only contains permission bits.
  if unsafe { libc::mkfifo(path.as_ptr(), 0o600) } == 0 {
    Ok(())
  } else {
    Err(std::io::Error::last_os_error())
  }
}

fn spawn_metadata_reader(
  app: tauri::AppHandle,
  metadata: File,
  album_art: PathBuf,
  stop: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
  std::thread::spawn(move || {
    let mut reader = BufReader::new(metadata);
    while !stop.load(Ordering::Relaxed) {
      match read_metadata_item(&mut reader) {
        Ok(Some(item)) => {
          listen_to_shairport_metadata(app.clone(), &item.kind, &item.code, item.data, &album_art)
        }
        Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
        Err(error) => {
          log_output(app.clone(), format!("Failed to read Shairport metadata: {error}"))
        }
      }
    }
  })
}

fn spawn_log_reader(
  app: tauri::AppHandle,
  stream: impl std::io::Read + Send + 'static,
  prefix: &'static str,
) {
  std::thread::spawn(move || {
    for line in BufReader::new(stream).lines() {
      match line {
        Ok(line) if !line.is_empty() => log_output(app.clone(), format!("{prefix} {line}")),
        Ok(_) => {}
        Err(error) => {
          log_output(app.clone(), format!("Error reading Shairport Sync output: {error}"));
          break;
        }
      }
    }
  });
}

#[derive(Debug, PartialEq)]
struct MetadataItem {
  kind: String,
  code: String,
  data: Vec<u8>,
}

fn read_metadata_item(reader: &mut impl BufRead) -> std::io::Result<Option<MetadataItem>> {
  let mut header = String::new();
  if reader.read_line(&mut header)? == 0 {
    return Ok(None);
  }
  let Some(kind) = xml_value(&header, "type").and_then(hex_code) else { return Ok(None) };
  let Some(code) = xml_value(&header, "code").and_then(hex_code) else { return Ok(None) };
  let length = xml_value(&header, "length").and_then(|value| value.parse().ok()).unwrap_or(0);
  if length == 0 {
    return Ok(Some(MetadataItem { kind, code, data: Vec::new() }));
  }

  let mut opening = String::new();
  reader.read_line(&mut opening)?;
  if opening.trim() != "<data encoding=\"base64\">" {
    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "missing Base64 data tag"));
  }
  let mut encoded = String::new();
  loop {
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
      return Err(std::io::Error::new(
        std::io::ErrorKind::UnexpectedEof,
        "metadata item ended before its closing tag",
      ));
    }
    if let Some((data, _)) = line.split_once("</data></item>") {
      encoded.extend(data.chars().filter(|character| !character.is_ascii_whitespace()));
      break;
    }
    encoded.extend(line.chars().filter(|character| !character.is_ascii_whitespace()));
  }
  let data = STANDARD
    .decode(&encoded)
    .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
  if data.len() != length {
    return Err(std::io::Error::new(
      std::io::ErrorKind::InvalidData,
      "invalid metadata item length",
    ));
  }
  Ok(Some(MetadataItem { kind, code, data }))
}

fn xml_value<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
  let start_tag = format!("<{tag}>");
  let end_tag = format!("</{tag}>");
  let start = text.find(&start_tag)? + start_tag.len();
  let end = text[start..].find(&end_tag)? + start;
  Some(&text[start..end])
}

fn hex_code(value: &str) -> Option<String> {
  let number = u32::from_str_radix(value, 16).ok()?;
  String::from_utf8(number.to_be_bytes().to_vec()).ok()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_text_metadata() {
    let input = b"<item><type>636f7265</type><code>6d696e6d</code><length>4</length>\n<data encoding=\"base64\">\nU29uZw==</data></item>\n";
    let item = read_metadata_item(&mut &input[..]).unwrap().unwrap();
    assert_eq!(
      item,
      MetadataItem { kind: "core".into(), code: "minm".into(), data: b"Song".to_vec() }
    );
  }

  #[test]
  fn parses_wrapped_metadata() {
    let input = b"<item><type>636f7265</type><code>6d696e6d</code><length>12</length>\n<data encoding=\"base64\">\nSGVsbG8s\nIHdvcmxk\n</data></item>\n";
    let item = read_metadata_item(&mut &input[..]).unwrap().unwrap();
    assert_eq!(
      item,
      MetadataItem { kind: "core".into(), code: "minm".into(), data: b"Hello, world".to_vec() }
    );
  }

  #[test]
  fn parses_empty_metadata() {
    let input = b"<item><type>73736e63</type><code>70626567</code><length>0</length></item>\n";
    let item = read_metadata_item(&mut &input[..]).unwrap().unwrap();
    assert_eq!(item, MetadataItem { kind: "ssnc".into(), code: "pbeg".into(), data: Vec::new() });
  }
}
