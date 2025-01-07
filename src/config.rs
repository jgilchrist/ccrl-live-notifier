use crate::ccrllive::CcrlLiveRoom;
use crate::cli::CliOptions;
use anyhow::Result;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;

pub struct Config {
    pub notify_webhook: String,
    pub log_webhook: Option<String>,

    pub rooms: Vec<CcrlLiveRoom>,
    pub engines: HashMap<String, HashSet<String>>,
}

#[derive(Deserialize)]
struct ConfigFile {
    pub rooms: Vec<String>,

    pub users: HashMap<String, HashSet<String>>,
}

pub fn get_config(cli_options: CliOptions) -> Result<Config> {
    let config_file =
        serde_json::from_reader::<File, ConfigFile>(File::open(&cli_options.config)?)?;

    let mut engines_to_users: HashMap<String, HashSet<String>> = HashMap::new();

    for (user, engines) in &config_file.users {
        for engine in engines {
            engines_to_users
                .entry(engine.clone())
                .or_default()
                .insert(user.clone());
        }
    }

    Ok(Config {
        notify_webhook: cli_options.notify_webhook.clone(),
        log_webhook: cli_options.log_webhook.clone(),
        rooms: config_file
            .rooms
            .iter()
            .map(|r| CcrlLiveRoom::new(r.as_str()))
            .collect(),
        engines: engines_to_users,
    })
}
