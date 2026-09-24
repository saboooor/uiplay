use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{Manager, path::BaseDirectory};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
  #[default]
  Shairport,
  Uxplay,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Settings {
  pub name: String,
  pub provider: Provider,
}

impl Default for Settings {
  fn default() -> Self {
    Self { name: "UiPlay".into(), provider: Provider::Shairport }
  }
}

pub fn load(app: &tauri::AppHandle) -> Settings {
  let Ok(path) = settings_path(app) else { return Settings::default() };
  fs::read_to_string(path)
    .ok()
    .and_then(|contents| serde_json::from_str(&contents).ok())
    .unwrap_or_default()
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Settings {
  load(&app)
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
  let path = settings_path(&app)?;
  let contents = serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?;
  fs::write(path, contents).map_err(|error| error.to_string())
}

fn settings_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
  app
    .path()
    .resolve("uiplay/settings.json", BaseDirectory::Config)
    .map_err(|error| error.to_string())
}
