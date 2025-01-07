use std::collections::HashSet;
use std::time::Duration;
use crate::notify::{NotifyContent};
use anyhow::Result;
use crate::log::Logger;
use crate::state::SeenGames;

mod ccrl_pgn;
mod config;
mod notify;
mod cli;
mod ccrllive;
mod log;
mod discord;
mod state;


const POLL_DELAY: Duration = Duration::from_secs(30);

fn main() -> Result<()> {
    let cli_options = cli::get_cli_options().expect("Unable to parse CLI");
    let config = config::get_config(cli_options).expect("Unable to load config");
    let log = log::get_logger(&config);

    let mut seen_games = SeenGames::load();

    loop {
        let current_games = ccrllive::get_current_games(&config, &log);

        let new_games = current_games.iter()
            // Filter out games we've already seen.
            .filter(|(_, game)| !seen_games.contains(game))
            .collect::<Vec<_>>();

        for (room, game) in &new_games {
            log.info(&format!("[{}] Saw game: {} vs {}", room.code(), &game.white_player, &game.black_player));

            let mut mentions = HashSet::new();

            for (engine, notifies) in &config.engines {
                if game.has_player(engine) {
                    mentions.extend(notifies.iter().cloned());
                    log.info(&format!("[{}] Saw engine: {} - NOTIFYING {} users", room.code(), &engine, notifies.len()));
                }
            }

            let notify_result = notify::notify(&config, NotifyContent {
                white_player: game.white_player.clone(),
                black_player: game.black_player.clone(),
                room: room.clone(),
                mentions,
            });

            if let Err(e) = notify_result {
                log.error(&format!("Unable to send notify: {:?}", e));
            }

            seen_games.add(game);
        }

        std::thread::sleep(POLL_DELAY);
    }
}
