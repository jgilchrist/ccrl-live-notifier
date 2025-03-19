use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct CliOptions {
    #[clap(long)]
    pub config_url: String,

    #[clap(long)]
    pub notify_webhook: String,

    #[clap(long)]
    pub log_webhook: Option<String>,
}

pub fn get_cli_options() -> Result<CliOptions> {
    Ok(CliOptions::parse())
}
