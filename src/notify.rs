use std::collections::HashSet;
use crate::config::Config;
use serde_json::json;
use crate::ccrllive::{CcrlLivePlayer, CcrlLiveRoom};
use anyhow::Result;

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

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Color::White => "White",
            Color::Black => "Black",
        })
    }
}

pub fn notify(config: &Config, content: NotifyContent) -> Result<()> {
    let client = reqwest::blocking::Client::new();

    let title = format!("{} started a game playing as {} vs. {}", content.engine, content.color, content.opponent);
    let description = format!("Watch live: {}\ncc. {}", content.room.url(), content.mentions.iter().map(|m| format!("<@!{}>", m)).collect::<Vec<_>>().join(" "));

    let body = json!({
        "username": "ccrl-live-notifier",
        "allowed_mentions": { "parse": ["users"] },
        "embeds": [{
            "title": title,
            "description": description,
        }]
    });

    client.post(&config.webhook_url)
        .json(&body)
        .send()?
        .error_for_status()?;

    Ok(())
}


