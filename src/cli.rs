use clap::Parser;
use crate::config::Config;
use anyhow::Result;
use crate::ccrllive::CcrlLiveRoom;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct CliOptions {
    #[clap(long)]
    pub discord_webhook: String,

    #[clap(long, required = true)]
    pub engines: Vec<String>,

    #[clap(long, required = true)]
    pub rooms: Vec<String>,
}

pub fn get_config_from_cli() -> Result<Config> {
    let cli = CliOptions::parse();

    Ok(Config {
        webhook_url: cli.discord_webhook,
        rooms: cli.rooms.iter().map(|r| CcrlLiveRoom::new(r.as_str())).collect(),
        engines: cli.engines,
    })
}
