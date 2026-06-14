use discord_rich_presence::{DiscordIpcClient, activity};
use std::sync::{Arc, LazyLock, Mutex};

pub struct DiscordState<'a> {
  pub client: DiscordIpcClient,
  pub activity: activity::Activity<'a>,
}

pub type SharedDiscordState<'a> = Arc<Mutex<Option<DiscordState<'a>>>>;
pub static DISCORD_STATE: LazyLock<SharedDiscordState> =
  LazyLock::new(|| Arc::new(Mutex::new(None)));