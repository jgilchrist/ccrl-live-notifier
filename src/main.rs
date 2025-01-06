use crate::ccrl_pgn::Pgn;
use std::collections::HashSet;
use std::time::Duration;
use crate::ccrllive::CcrlLiveRoom;
use crate::config::Config;
use crate::notify::{Color, NotifyContent};

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

        pgns.push((room.clone(), pgn));
    }

    pgns
}

fn main() -> ! {
    let poll_delay = Duration::from_secs(30);
    let config = cli::get_config_from_cli().unwrap();

    let mut seen_games = HashSet::<Pgn>::new();

    loop {
        let current_games = get_current_games(&config);

        let new_games = current_games.iter()
            // Don't look at games until they're out of book. If we catch one while it's still playing
            // book moves, we'll get a different hash for this game later.
            .filter(|(_, game)| game.out_of_book())
            // Filter out games we've already seen.
            .filter(|(_, game)| !seen_games.contains(game))
            .collect::<Vec<_>>();


        for (room, game) in &new_games {
            println!("[{}] Saw game: {} vs {}", room.code(), &game.white_player, &game.black_player);

            // FIXME: If watching for both engines, don't notify twice
            for engine in &config.engines {
                if game.white_player_is(engine) {
                    println!("[{}] Saw engine as white: {} - NOTIFYING", room.code(), &engine);

                    notify::notify(&config, NotifyContent {
                        engine: game.white_player.clone(),
                        opponent: game.black_player.clone(),
                        color: Color::White,
                        room: room.clone(),
                    })
                }

                if game.black_player_is(engine) {
                    println!("[{}] Saw engine as black: {} - NOTIFYING", room.code(), &engine);

                    notify::notify(&config, NotifyContent {
                        engine: game.black_player.clone(),
                        opponent: game.white_player.clone(),
                        color: Color::Black,
                        room: room.clone(),
                    })
                }
            }

            seen_games.insert(game.clone());
        }

        std::thread::sleep(poll_delay);
    }
}
