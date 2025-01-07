use crate::ccrllive::{CcrlLivePlayer, CcrlLiveRoom};
use crate::config::Config;
use crate::discord;
use anyhow::Result;
use std::collections::HashSet;

pub struct NotifyContent {
    pub white_player: CcrlLivePlayer,
    pub black_player: CcrlLivePlayer,
    pub room: CcrlLiveRoom,
    pub mentions: HashSet<String>,
}

pub fn notify(config: &Config, content: NotifyContent) -> Result<()> {
    discord::send_embed_message(
        &config.notify_webhook,
        &format!(
            ":white_medium_square: {} vs. :black_medium_square: {} starting",
            content.white_player, content.black_player
        ),
        &format!(
            "Watch live: {}\ncc. {}",
            content.room.url(),
            content
                .mentions
                .iter()
                .map(|m| format!("<@!{}>", m))
                .collect::<Vec<_>>()
                .join(" ")
        ),
    )
}
