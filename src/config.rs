use std::collections::{HashMap, HashSet};
use std::fs::File;
use serde::Deserialize;
use crate::cli::CliOptions;
use anyhow::Result;
use crate::ccrllive::CcrlLiveRoom;

pub struct Config {
    pub webhook_url: String,

    pub rooms: Vec<CcrlLiveRoom>,
    pub engines: HashMap<String, HashSet<String>>,
}

#[derive(Deserialize)]
struct ConfigFile {
    pub rooms: Vec<String>,

    pub users: HashMap<String, HashSet<String>>,
}

pub fn get_config(cli_options: CliOptions) -> Result<Config> {
    let config_file = serde_json::from_reader::<File, ConfigFile>(File::open(&cli_options.config)?)?;

    let mut engines_to_users: HashMap<String, HashSet<String>> = HashMap::new();

    for (user, engines) in &config_file.users {
        for engine in engines {
            engines_to_users.entry(engine.clone()).or_insert(HashSet::new()).insert(user.clone());
        }
    }

    Ok(Config {
        webhook_url: cli_options.discord_webhook.clone(),
        rooms: config_file.rooms.iter().map(|r| CcrlLiveRoom::new(r.as_str())).collect(),
        engines: engines_to_users,
    })
}