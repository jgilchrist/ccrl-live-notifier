use std::collections::HashSet;
use crate::config::Config;
use crate::ccrllive::{CcrlLivePlayer, CcrlLiveRoom};
use anyhow::Result;
use crate::discord;

pub struct NotifyContent {
    pub engine: CcrlLivePlayer,
    pub opponent: CcrlLivePlayer,
    pub color: Color,
    pub room: CcrlLiveRoom,
    pub mentions: HashSet<String>,
}

pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn other(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Color::White => ":white_medium_square:",
            Color::Black => ":black_medium_square:",
        })
    }
}

pub fn notify(config: &Config, content: NotifyContent) -> Result<()> {
    discord::send_embed_message(
        &config.notify_webhook,
        &format!("{} {} vs. {} {} starting", content.color, content.engine, content.color.other(), content.opponent),
        &format!("Watch live: {}\ncc. {}", content.room.url(), content.mentions.iter().map(|m| format!("<@!{}>", m)).collect::<Vec<_>>().join(" ")),
    )
}


