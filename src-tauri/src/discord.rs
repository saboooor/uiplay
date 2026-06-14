use discord_rich_presence::{DiscordIpcClient, activity};
use std::sync::{Arc, LazyLock, Mutex};

pub struct DiscordState<'a> {
  pub client: DiscordIpcClient,
  pub activity: activity::Activity<'a>,
}

pub type SharedDiscordState<'a> = Arc<Mutex<Option<DiscordState<'a>>>>;
pub static DISCORD_STATE: LazyLock<SharedDiscordState> =
  LazyLock::new(|| Arc::new(Mutex::new(None)));

pub type SharedActivityAssets<'a> = Arc<Mutex<activity::Assets<'a>>>;
pub static ACTIVITY_ASSETS: LazyLock<SharedActivityAssets> = LazyLock::new(|| 
  Arc::new(Mutex::new(activity::Assets::new()
    .large_image("uiplay")
    .large_text("UiPlay")
    .small_image("uiplay")
    .small_text("UiPlay")
  ))
);