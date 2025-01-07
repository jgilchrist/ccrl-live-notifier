use crate::log::Logger;
use crate::notify::NotifyContent;
use crate::state::SeenGames;
use anyhow::Result;
use std::collections::HashSet;
use std::time::Duration;

mod ccrl_pgn;
mod ccrllive;
mod cli;
mod config;
mod discord;
mod log;
mod notify;
mod state;

const POLL_DELAY: Duration = Duration::from_secs(30);

fn main() -> Result<()> {
    let cli_options = cli::get_cli_options().expect("Unable to parse CLI");
    let config = config::get_config(cli_options).expect("Unable to load config");
    let log = log::get_logger(&config);

    std::panic::set_hook(Box::new(|info| {
        // FIXME: Lifetimes mean we need to re-do this initialisation in the panic handler.
        let cli_options = cli::get_cli_options().unwrap();
        let config = config::get_config(cli_options).unwrap();
        let log = log::get_logger(&config);
        log.panic(info);
    }));

    let mut first_run = true;

    let mut seen_games = SeenGames::load().expect("Unable to load state");

    log.info("```───────────────────────────────────────────────────────────────────────────────────────────────────────────```");

    loop {
        let current_games = ccrllive::get_current_games(&config, &log);

        if first_run {
            for (room, game) in &current_games {
                log.info(&format!(
                    "`{}` In progress: `{}` vs `{}` ({} plies)",
                    room.code(),
                    game.white_player,
                    game.black_player,
                    game.moves.len()
                ))
            }

            first_run = false;
        }

        let new_games = current_games
            .iter()
            // Filter out games we've already seen.
            .filter(|(_, game)| !seen_games.contains(game))
            .collect::<Vec<_>>();

        for (room, game) in &new_games {
            log.info(&format!(
                "`{}` New game: `{}` vs `{}`",
                room.code(),
                &game.white_player,
                &game.black_player
            ));

            let mut mentions = HashSet::new();

            for (engine, notifies) in &config.engines {
                if game.has_player(engine) {
                    mentions.extend(notifies.iter().cloned());
                    log.info(&format!(
                        "`{}` Will notify {} users for engine `{}`",
                        room.code(),
                        notifies.len(),
                        &engine,
                    ));
                }
            }

            if !mentions.is_empty() {
                let notify_result = notify::notify(
                    &config,
                    NotifyContent {
                        white_player: game.white_player.clone(),
                        black_player: game.black_player.clone(),
                        room: room.clone(),
                        mentions,
                    },
                );

                if let Err(e) = notify_result {
                    log.error(&format!("Unable to send notify: {:?}", e));
                }
            }

            let write_state_result = seen_games.add(game);

            if let Err(e) = write_state_result {
                log.error(&format!("Unable to write seen game to file: {:?}", e));
            }
        }

        std::thread::sleep(POLL_DELAY);
    }
}
