use crate::ccrl_pgn;
use crate::ccrl_pgn::Pgn;
use crate::log::Logger;
use anyhow::Result;
use std::fmt::Formatter;
use std::hash::Hasher;

const CCRL_LIVE_ROOMS_URL: &str = "https://ccrl.live/broadcasts";

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
        if self
            .name
            .to_ascii_lowercase()
            .contains(&search.to_ascii_lowercase())
        {
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

fn get_active_broadcasts() -> Result<Vec<CcrlLiveRoom>> {
    let response = reqwest::blocking::get(CCRL_LIVE_ROOMS_URL)?.error_for_status()?;

    let rooms = response
        .json::<Vec<u64>>()?
        .iter()
        .map(|r| CcrlLiveRoom::new(&r.to_string()))
        .collect();

    Ok(rooms)
}

fn get_current_pgn(room: &CcrlLiveRoom) -> Result<Option<Pgn>> {
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

pub fn get_current_games(log: &dyn Logger) -> Result<Vec<(CcrlLiveRoom, Pgn)>> {
    let mut pgns: Vec<(CcrlLiveRoom, Pgn)> = vec![];

    let broadcasts = get_active_broadcasts()?;

    for room in &broadcasts {
        let pgn_fetch_result = get_current_pgn(room);

        let Ok(pgn) = pgn_fetch_result else {
            let e = pgn_fetch_result.unwrap_err();

            log.warning(&format!(
                "Unable to fetch PGN for room {}: {:?}",
                room.code(),
                e
            ));

            continue;
        };

        // We may have no PGN for the room if there's no active broadcast
        let Some(pgn) = pgn else {
            continue;
        };

        // Don't consider games which are still in book to have started since we need all the book
        // moves so we can hash the game correctly
        if !pgn.out_of_book() {
            continue;
        }

        pgns.push((room.clone(), pgn));
    }

    Ok(pgns)
}
