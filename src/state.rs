use crate::ccrl_pgn::Pgn;
use std::collections::HashSet;

pub struct SeenGames(HashSet<Pgn>);

impl SeenGames {
    pub fn load() -> Self {
        SeenGames(HashSet::new())
    }

    pub fn contains(&self, game: &Pgn) -> bool {
        self.0.contains(&game)
    }

    pub fn add(&mut self, game: &Pgn) -> bool {
        self.0.insert(game.clone())
    }
}
