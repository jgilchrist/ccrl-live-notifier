use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct CliOptions {
    #[clap(long)]
    pub discord_webhook: String,

    #[clap(long, default_value = "config.json")]
    pub config: PathBuf,
}

pub fn get_cli_options() -> Result<CliOptions> {
    Ok(CliOptions::parse())
}
