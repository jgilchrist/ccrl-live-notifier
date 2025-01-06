use std::fmt::Formatter;
use std::hash::Hasher;
use crate::ccrl_pgn::Pgn;
use anyhow::Result;
use crate::ccrl_pgn;

#[derive(Debug, Clone)]
pub struct CcrlLiveRoom {
    code: String,
}

impl CcrlLiveRoom {
    pub fn new(code: &str) -> Self {
        Self { code: code.into() }
    }

    pub fn code(&self) -> String {
        self.code.clone()
    }

    fn ccrl_url(suffix: &str) -> String {
        format!("https://ccrl.live/{suffix}")
    }

    pub fn url(&self) -> String {
        Self::ccrl_url(&self.code)
    }

    pub fn pgn_url(&self) -> String {
        Self::ccrl_url(&format!("{}/pgn", self.code))
    }
}

#[derive(Debug, Clone)]
pub struct CcrlLivePlayer {
    name: String,
}

impl CcrlLivePlayer {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn matches(&self, search: &str) -> bool {
        if self.name.to_ascii_lowercase().contains(&search.to_ascii_lowercase()) {
            return true;
        }

        // TODO: Normalisation (i.e. remove 64-bit)
        false
    }
}

impl std::fmt::Display for CcrlLivePlayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::hash::Hash for CcrlLivePlayer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

pub fn get_current_pgn(room: &CcrlLiveRoom) -> Result<Option<Pgn>> {
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let response = client.get(room.pgn_url()).send()?.error_for_status()?;

    if response.status() != reqwest::StatusCode::OK {
        return Ok(None);
    }

    let pgn_content = response.text()?;

    let pgn_info = ccrl_pgn::get_pgn_info(&pgn_content)?;

    Ok(Some(pgn_info))
}