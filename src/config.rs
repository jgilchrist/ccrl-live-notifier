use crate::cli::CliOptions;
use anyhow::Result;
use reqwest::Url;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct NotifyConfig {
    pub engines: HashMap<String, HashSet<String>>,
}

pub struct Config {
    pub config_url: Url,
    pub notify_webhook: String,
    pub log_webhook: Option<String>,
}

#[derive(Deserialize)]
struct ConfigFile {
    pub users: HashMap<String, HashSet<String>>,
}

pub fn get_config(cli_options: CliOptions) -> Result<Config> {
    Ok(Config {
        config_url: Url::parse(&cli_options.config_url)?,
        notify_webhook: cli_options.notify_webhook.clone(),
        log_webhook: cli_options.log_webhook.clone(),
    })
}

pub fn get_notify_config(config: &Config) -> Result<NotifyConfig> {
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let response = client
        .get(config.config_url.clone())
        .send()?
        .error_for_status()?;

    let config_file_contents = response.text()?;

    let config_file = serde_json5::from_str::<ConfigFile>(&config_file_contents)?;

    let mut engines_to_users: HashMap<String, HashSet<String>> = HashMap::new();

    for (user, engines) in &config_file.users {
        for engine in engines {
            engines_to_users
                .entry(engine.clone())
                .or_default()
                .insert(user.clone());
        }
    }

    Ok(NotifyConfig {
        engines: engines_to_users,
    })
}
