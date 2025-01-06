use crate::ccrl_pgn::Pgn;
use std::collections::HashSet;
use std::time::Duration;
use crate::ccrllive::CcrlLiveRoom;
use crate::config::Config;
use crate::notify::{Color, NotifyContent};
use anyhow::Result;

mod ccrl_pgn;
mod config;
mod notify;
mod cli;
mod ccrllive;

fn get_current_games(config: &Config) -> Vec<(CcrlLiveRoom, Pgn)> {
    let mut pgns: Vec<(CcrlLiveRoom, Pgn)> = vec![];

    for room in &config.rooms {
        let Ok(pgn) = ccrllive::get_current_pgn(room) else {
            // TODO: Log error

            continue;
        };

        // Don't consider games which are still in book to have started since we need all the book
        // moves so we can hash the game correctly
        if !pgn.out_of_book() {
            continue;
        }

        pgns.push((room.clone(), pgn));
    }

    pgns
}

const POLL_DELAY: Duration = Duration::from_secs(30);

fn main() -> Result<()> {
    let cli_options = cli::get_cli_options().expect("Unable to parse CLI");
    let config = config::get_config(cli_options).expect("Unable to load config");

    let mut seen_games = HashSet::<Pgn>::new();

    loop {
        let current_games = get_current_games(&config);

        let new_games = current_games.iter()
            // Filter out games we've already seen.
            .filter(|(_, game)| !seen_games.contains(game))
            .collect::<Vec<_>>();

        for (room, game) in &new_games {
            println!("[{}] Saw game: {} vs {}", room.code(), &game.white_player, &game.black_player);

            // FIXME: If watching for both engines, don't notify twice
            for (engine, notifies) in &config.engines {
                if game.white_player_is(engine) {
                    println!("[{}] Saw engine as white: {} - NOTIFYING {} users", room.code(), &engine, notifies.len());

                    notify::notify(&config, NotifyContent {
                        engine: game.white_player.clone(),
                        opponent: game.black_player.clone(),
                        color: Color::White,
                        room: room.clone(),
                        mentions: notifies.clone(),
                    })
                }

                if game.black_player_is(engine) {
                    println!("[{}] Saw engine as black: {} - NOTIFYING {} users", room.code(), &engine, notifies.len());

                    notify::notify(&config, NotifyContent {
                        engine: game.black_player.clone(),
                        opponent: game.white_player.clone(),
                        color: Color::Black,
                        room: room.clone(),
                        mentions: notifies.clone(),
                    })
                }
            }

            seen_games.insert(game.clone());
        }

        std::thread::sleep(POLL_DELAY);
    }
}
