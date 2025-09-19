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
    let mentions_str = if !content.mentions.is_empty() {
        "   cc. ".to_string()
            + content
                .mentions
                .iter()
                .map(|m| format!("<@!{}>", m))
                .collect::<Vec<_>>()
                .join(" ")
                .as_str()
    } else {
        String::new()
    };

    discord::send_message(
        &config.notify_webhook,
        &format!(
            "[`{}`]({}) `{}` vs. `{}`{}",
            content.room.code(),
            content.room.url(),
            content.white_player,
            content.black_player,
            mentions_str
        ),
    )
}
