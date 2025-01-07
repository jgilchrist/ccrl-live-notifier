use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct CliOptions {
    #[clap(long)]
    pub notify_webhook: String,

    #[clap(long)]
    pub log_webhook: Option<String>,

    #[clap(long, default_value = "config.json")]
    pub config: PathBuf,
}

pub fn get_cli_options() -> Result<CliOptions> {
    Ok(CliOptions::parse())
}
