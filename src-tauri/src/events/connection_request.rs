use crate::listen::DEVICE_ID;

pub fn connection_request(caps: regex::Captures<'_>) {
  let device_id = caps.get(2).map_or("", |m| m.as_str()).to_string();
  if let Ok(mut guard) = DEVICE_ID.lock() {
    *guard = device_id;
  }}