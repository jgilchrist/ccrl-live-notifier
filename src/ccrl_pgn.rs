use std::hash::{Hash, Hasher};
use pgn_reader::{BufferedReader, RawComment, RawHeader, SanPlus, Skip, Visitor};
use anyhow::Result;
use crate::ccrllive::{CcrlLivePlayer};

const WHITE_HEADER_KEY: &str = "White";
const BLACK_HEADER_KEY: &str = "Black";
const DATE_HEADER_KEY: &str = "Date";
const BOOK_MOVE_COMMENT_VALUE: &str = "(Book)";

#[derive(Debug, Clone)]
pub struct Pgn {
    pub white_player: CcrlLivePlayer,
    pub black_player: CcrlLivePlayer,
    pub date: String,

    pub moves: Vec<(String, bool)>,
}

impl Pgn {
    /// The game is 'out of book' if any of the moves that were played are not book moves
    pub fn out_of_book(&self) -> bool {
        self.moves.iter().any(|(_, in_book)| !in_book)
    }

    pub fn white_player_is(&self, player: &str) -> bool {
        self.white_player.matches(player)
    }

    pub fn black_player_is(&self, player: &str) -> bool {
        self.black_player.matches(player)
    }
}

// The hash of a CCRL PGN is the hash of the players, the date, and the book.
// That is to say, we consider games equivalent if they are played by the same players
// on the same day, with the same opening book.
// FIXME: This doesn't account for replays.
impl Hash for Pgn {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.white_player.hash(state);
        self.black_player.hash(state);
        self.date.hash(state);

        let book_moves = self.moves.iter().filter(|(_, in_book)| *in_book);

        for (mv, _) in book_moves {
            mv.hash(state);
        }
    }
}

fn hash(v: impl Hash) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish()
}

impl PartialEq<Self> for Pgn {
    fn eq(&self, other: &Self) -> bool {
        hash(self) == hash(other)
    }
}

impl Eq for Pgn {}

struct PgnInfoBuilder {
    pub white_player: Option<String>,
    pub black_player: Option<String>,
    pub date: Option<String>,
    pub moves: Vec<(String, bool)>,

    pub last_san: Option<String>,
}

impl PgnInfoBuilder {
    pub fn new() -> PgnInfoBuilder {
        Self {
            white_player: None,
            black_player: None,
            date: None,
            moves: vec![],

            last_san: None,
        }
    }
}

impl Visitor for PgnInfoBuilder {
    type Result = Pgn;

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        let key = String::from_utf8_lossy(key);
        let value = value.decode_utf8_lossy();

        if key == WHITE_HEADER_KEY {
            self.white_player = Some(value.to_string());
        }

        if key == BLACK_HEADER_KEY {
            self.black_player = Some(value.to_string());
        }

        if key == DATE_HEADER_KEY {
            self.date = Some(value.to_string());
        }
    }

    fn san(&mut self, san: SanPlus) {
        assert_eq!(self.last_san, None);

        self.last_san = Some(san.to_string());
    }

    fn comment(&mut self, comment: RawComment<'_>) {
        let Some(san) = self.last_san.clone() else {
            panic!("Saw PGN comment without preceding SAN");
        };

        let comment = String::from_utf8_lossy(comment.as_bytes()).to_string();
        let is_book_move = comment == BOOK_MOVE_COMMENT_VALUE;

        self.moves.push((san, is_book_move));
        self.last_san = None;
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true)
    }

    fn end_game(&mut self) -> Self::Result {
        assert_ne!(self.white_player, None);
        assert_ne!(self.black_player, None);
        assert_ne!(self.date, None);
        assert_ne!(self.moves.len(), 0);

        Pgn {
            white_player: CcrlLivePlayer::new(self.white_player.clone().unwrap()),
            black_player: CcrlLivePlayer::new(self.black_player.clone().unwrap()),
            date: self.date.clone().unwrap(),
            moves: self.moves.clone(),
        }
    }
}

pub fn get_pgn_info(pgn: &str) -> Result<Pgn> {
    let mut reader = BufferedReader::new_cursor(pgn);

    let pgn_info = reader.read_game(&mut PgnInfoBuilder::new()).expect("Unable to parse PGN").unwrap();

    Ok(pgn_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pgn_parsing_grabs_correct_information() {
        let sample_pgn = r#"[Site "114th Amateur D11"]
[Date "2025.01.06"]
[White "RookieMonster 1.9.9 64-bit"]
[Black "Betsabe_II 2023"]

1. d4 {(Book)} Nf6 {(Book)} 2. c4 {(Book)} g6 {(Book)} 3. Nf3 {(Book)} Bg7 {(Book)} 4. Nc3 {(Book)} d5 {(Book)} 5. Qa4+ {(Book)} Bd7 {(Book)} 6. Qb3 {(Book)} dxc4 {(Book)} 7. Qxc4 {(Book)} O-O {(Book)} 8. Bf4 {(Book)} Na6 {(Book)} 9. e4 {(e4 c5 d5 b5 Nxb5 Nxe4 Qxe4 Bxb5 O-O-O Qa5 Bxb5 Qxb5 Be5 Bxe5 Nxe5 Rab8 Nc4 Nb4) -0.41/18 1736170414} c5 {(c5) -0.57/17 38} 10. Be2 {(Be2 cxd4 Qxd4 Bc6 O-O Nxe4 Qe3 Nxc3 bxc3 Qa5 Ne5 Nc5 Rad1 Rac8 Bc4 Be4 Rfe1) -0.39/17 33} cxd4 {(cxd4 Qxd4 Bc6 e5 Nh5 Qxd8 Raxd8 Bg5 f6 exf6 Bxf6 Bh6) 0.22/17 22} 11. Qxd4 {(Qxd4 Bc6 e5 Nh5 Be3 Qxd4 Nxd4 Bxg2 Rg1 Bh3 O-O-O Nc5 Bxh5 gxh5 Kb1 Rad8 Rg5 Ne6 Rxh5 Nxd4) -0.68/20 28} Bc6 {(Bc6) 0.35/18 27} 12. e5 {(e5 Nh5 Be3 Qxd4 Nxd4 Bxg2 Rg1 Bh3 Bxh5 gxh5 Bh6 Bg4 Nf5 Bxe5 Nxe7+ Kh8 Bxf8 Rxf8 h3 Bxh3) -0.26/19 20} Ng4 {(Ng4 O-O-O Qc7 Bg3 Bxf3 Bxf3 Nxe5 Qe3 e6 Rhe1 f6 Rd2) 0.13/18 17} 13. Rd1 {(Rd1 Bxf3 Bxf3 Qxd4 Rxd4 Nxe5 Bxe5 Bxe5 Rc4 Bxc3+ bxc3 Rfd8 O-O Rab8 Re1 e6 a3 Rd2 Rd4 Rxd4 cxd4 Nc7) -0.71/21 36} Qa5 {(Qa5 Bg3 Bxf3 Bxf3 Nxe5 Bxe5 Bxe5 Qe3 Bxc3+ bxc3 Nc5 O-O) 0.64/19 32} 14. O-O {(O-O e6 Ne4 Rad8 Nd6 Bxf3 Bxf3 Nxe5 Qe3 Nxf3+ Qxf3 Qb4 Nxb7 Rxd1 Rxd1 Qxb2 Nd6 Nc5) -0.30/17 19} e6 {(e6 Qd6 Rfd8 Qe7 Rxd1 Rxd1 Qb6 Nd4 Rd8 Qxd8+ Qxd8 Nxc6) 0.17/18 50} 15. Qd6 {(Qd6 Bxf3 Bxf3 Rad8 Qe7 Nxe5 Bxb7 Nb4 Be4 Nbc6 Qg5 Qb6 Na4 Qb4 Bxe5 Qxe4 Bxg7) -0.04/17 34} Rad8 {(Rad8) 0.02/17 21} 16. Qe7 {(Qe7 Bxf3 Bxf3 Nxe5 Bxb7 Nb4 Be4 Nbc6) -0.04/15 18} Qb6 {(Qb6) 0.32/17 16} 17. Rxd8 {(Rxd8 Qxd8 Qd6 Qb6 h3 Rd8 Qe7 Re8 Qg5 Qxb2 Bxa6 Qxc3 Rc1 Qa5 Rxc6 bxc6 Bb7 Qb6) 0.07/18 18} Qxd8 {(Qxd8 Qxd8 Rxd8 Bxa6 bxa6 Re1 Bf8 h3 Nh6 Bg5 Rd3 Rd1) 0.27/20 21} 18. Qd6 {(Qd6 Qb6 h3 Rd8 Qe7 Re8 Qd6 Rd8) 0.02/19 18} Qb6 {(Qb6) 0.36/19 15} 19. h3 {(h3 Rd8 Qe7 Re8 Qd6 Rd8) 0.02/18 18} Rd8 {(Rd8) 0.24/19 15} 20. Qe7 {(Qe7 Re8 Qd6 Rd8) 0.02/20 17} Rd7 {(Rd7) 0.56/19 26} 21. Qa3 {(Qa3 Bxf3 hxg4 Bxe2 Nxe2 Bf8 Qc3 Bc5 Bg5 Nb4 Nc1 h6 Bd2 g5 Nb3 Nxa2 Qc4 Nb4 Bxb4 Bxb4) -1.53/20 26} Bxf3 {(Bxf3) 0.63/20 18} 22. hxg4 {(hxg4 Bxe2 Nxe2 Bf8 Qc3 Nb4 Be3 Nxa2 Qc2 Qc7 Qe4 Nb4 Ra1 Qc6 Ng3 a6 Rc1 Qxe4 Nxe4 Nd5 Bg5 h6 Bf6) -1.56/23 33} Bxe2 {(Bxe2 Nxe2 Bf8 Qc3 Nc7 Be3 Qa6 Nc1 Nd5 Qd4 Nxe3 Qxe3) 0.99/19 21} 23. Nxe2 {(Nxe2 Bf8 Qc3 Nb4 Bg5 Rc7 Qf3 Nc6 Qc3 Qb5 Re1 Qxe5 Qd2 Qd6 Qxd6 Bxd6 Nc3 Be5 Nb5 Rd7 Nxa7 Bxb2 Nxc6) -1.68/23 26} Bf8 {(Bf8) 0.99/19 13} 24. Qc3 {(Qc3 Nb4 Bg5 Rc7 Qf3 Nc6 Qc3 Qb5 Qc2 Qa5 Bf6 Nxe5 Qe4 Bg7 Bxg7 Kxg7 Rd1 Nd7 Nc3 Qc5 Qf4 g5) -1.77/22 31} Nc7 {(Nc7 Be3 Qa6 Nf4 Qxa2 Rc1 Qa6 b3 Bg7 g5 Qb5 Bd4) 0.92/18 19} 25. Be3 {(Be3 Qb5 Nf4 Bg7 Bc5 b6 Bd4 Qa4 Ne2 Qxa2 g5 Bf8 Qc6 Qd5 Qc2 Bg7 f4 Nb5 Rd1 Rc7) -1.37/20 30} Qb5 {(Qb5) 0.85/21 38} 26. Qc2 {(Qc2 Qxe5 Bxa7 Qa5 Bb8 Bg7 g5 Be5 Nc3 Rd8 Bxc7 Qxc7 Rd1 Rxd1+ Qxd1 Bxc3 bxc3 Qxc3 Qd8+ Kg7 Kh2 b5 g3) -1.34/21 29} Qxe5 {(Qxe5) 0.89/19 18} 27. Bxa7 {(Bxa7 Qa5 Bb8 Bg7 g5 Rd2 Qc4 Be5 Rc1) -1.24/20 38} Qa5 {(Qa5) 1.01/19 24} 28. Bb8 {(Bb8 Bg7 g5 Rd2 Qc4) -1.24/18 27} Rd2 {(Rd2 Bxc7 Qd5 Qb3 Qxb3 axb3 Rxe2 Rd1 Bc5 b4 Be7 Kf1) 0.81/18 19} 29. Qc3 {(Qc3 Bb4 Qxc7 Qxc7 Bxc7 Rxe2 Rc1 Rxb2 Be5 Rd2 Rb1 Bc5 Rxb7 Bxf2+ Kh2 Bc5 Rb8+ Bf8 a4 Rd5 Bc3 Rc5 Bf6) -1.08/21 17} Bb4 {(Bb4) 0.85/20 20} 30. Qxc7 {(Qxc7 Qxc7 Bxc7 Rxe2 Rc1 Bf8 Bg3 Rxb2 Rc8 Rd2 Rb8 b6 Rxb6 Rxa2 Rb8 Rd2 Kh2 h6 f3 Rd5 Rb7 Rd2) -0.87/22 17} Qxc7 {(Qxc7 Bxc7 Rxe2 Rc1 Bf8 Bf4 e5 Be3 Rxb2 a4 Ra2 Rc8) 0.80/20 17} 31. Bxc7 {(Bxc7 Rxe2 Rc1 Bf8 Bg3 Rxb2 Rc8 Rb6 Kh2 f6 a4 Ra6 Rb8 g5 Rxb7 Rxa4 Rb6 Re4 Rc6 Re2 Rb6 Re1 Rb7) -0.63/23 17} Rxe2 {(Rxe2 Rc1 Bf8 Bf4 e5 Bg3 Rxb2 Bxe5 Re2 Bf6 Rxa2 Rb1) 0.81/19 26} 32. Rc1 {(Rc1 Bf8 Bg3 Rxb2 Rc8 Rb6 Kh2 f6 a4 Kg7 Rc7+ Kh8 Rf7 Bg7 Rd7 h6 f3 Rb4 a5 Kh7 Re7 Rb5 Rxe6 Rxa5) -0.39/23 16} Bf8 {(Bf8) 0.68/19 22} 33. Bg3 {(Bg3 Rxb2 Rc8 Rb6 g5 Rb1+ Kh2 Rd1 Be5 b6 Rb8 Rd5 Bf6 Rd7 f3 Ra7 Rxb6 Rxa2 Rb8 Rc2 Rd8 Rc1 Rd7) 0.01/23 16} f6 {(f6) 0.59/18 18} 34. Rc8 {(Rc8 e5 b3 Rxa2 Rb8 h6 Rxb7 Rb2 Kh2 Bc5 f3 Rb1 Bh4 f5 gxf5 gxf5 Bf6 e4 fxe4 fxe4 Be5 h5) -0.12/22 16} e5 {(e5) 0.50/20 17} 35. b4 {(b4 Rxa2 Rb8 Kf7 Rxb7+ Ke6 b5 Rb2 Kh2 h6 Kh3 Bc5 f3 Bd4 Rb8 Rb1 Kh2 Bg1+) -0.11/21 16} Rxa2 {(Rxa2 Rb8 Kf7 Rxb7+ Ke6 b5 Rb2 Kh2 Bc5 f3 h6 Kh3) 0.54/20 21} 36. Rb8 {(Rb8 Kf7 Rxb7+ Ke6 b5 Rb2 Kh2 h6 Rb6+ Bd6 Rb7 Bc5 f3 Bd4 Rb8 f5 gxf5+ gxf5 Kh3 h5 Bh4 Rb1 Re8+ Kd5 Rd8+ Kc5) -0.11/21 15} Kf7 {(Kf7) 0.54/19 15} 37. Rxb7+ {(Rxb7+ Ke6 b5 Rb2 Kh2 h6 f3 Bc5 Rb8 Rb1 Re8+ Kf7 Rb8 Be3 Bh4 Rb2 Be1 Ke6) -0.12/21 15} Ke6 {(Ke6) 0.54/19 21} 38. b5 {(b5 Rb2 Kh2 h6 f3 Bc5 Rb8 Rb1 Re8+ Kf7 Rb8 Be3 Bh4 Bg1+ Kh3 Bc5 Rb7+ Ke6 Kh2 Bg1+ Kh3 Bd4 Rb8) -0.18/22 15} Rb2 {(Rb2) 0.57/19 20} 39. Rxh7 {(Rxh7 Rxb5) -0.11/17 15} Rxb5 {(Rxb5 Kh2 Bc5 Rh8 Bd4 Re8+ Kf7 Rc8 Rc5 Rxc5 Bxc5 f3) 0.98/18 13} 40. Kh2 {(Kh2 Rb2) -0.15/16 15} f5 {(f5 Rh8 Kf7 gxf5 gxf5 f3 e4 Bf4 Rb2 fxe4 fxe4 Rh5) 0.94/17 18} 41. Rh8 {(Rh8 Bg7 gxf5+ gxf5 Re8+ Kf7 Rc8 Bf6 Rc7+ Ke6 Rc6+ Ke7 f3 Rb2 Kh3 Kf7 Rc5 Ke6 Rc6+ Ke7 Kh2 Ra2 Rc5) -0.22/19 15} Bg7 {(Bg7) 0.93/19 28} 42. gxf5+ {(gxf5+ gxf5 Re8+ Kf7 Rc8 Bf6 Rc7+ Ke6 Rc6+ Ke7 f3 Rb2 Kh3 Rb1 Kh2 Kf7 Ra6 Rb2 Ra5) -0.23/19 14} gxf5 {(gxf5 Re8+ Kd5 Rd8+ Kc4 Rd7 Bf6 Rd6 Bg5 Kh3 e4 Rc6+) 0.88/17 11} 43. Re8+ {(Re8+ Kf7 Rc8 Bf6) -0.23/19 14} Kf7 {(Kf7) 0.94/18 12} 44. Rc8 {(Rc8 Bf6) -0.19/19 14} Bf8 {(Bf8) 0.89/17 16} 45. Rc6 {(Rc6 Be7 f3 e4 Rc7 Ke6 Bf4 Rb2 Kg1 Ra2 fxe4 fxe4 g4 Ra3 Rc8 e3) -0.45/18 14} Rd5 {(Rd5 Bh4 Bd6 g3 Rd2 Kg2 Ke6 Bg5 Rd3 Kh3 Kd5 Rc2) 0.93/18 19} 46. Bh4 {(Bh4 Bd6 Bg5 Kg7 g3 f4 gxf4 exf4 Rxd6 Rxd6 Bxf4) 0.00/20 14} Bd6 {(Bd6) 0.79/18 13} 47. Bg5 {(Bg5 Kg7 g3 f4 gxf4 exf4 Rxd6 Rxd6 Bxf4) 0.00/20 14} e4+ {(e4+) 0.69/17 10} 48. g3 {(g3 Be5 Be3 Rb5 Ra6 Rb2 Kh3 Rb1 Ra5 Ke6 Ra8 Kf7 Kg2 Ke6 Ra6+ Kd5 Ra5+ Kd6 Kh2 Ke6 Ra7 Rb2 Kh3 Bf6) 0.03/22 14} Be5 {(Be5) 0.78/18 10} 49. Be3 {(Be3 Rb5 Ra6 Rb2 Kh3 Rb1 Ra5 Ke6 Ra8 Rh1+ Kg2 Rb1 Ra6+ Kf7 Ra8 Ke6) 0.03/22 13} Rd8 {(Rd8) 0.50/18 13} 50. Kg2 {(Kg2 Rd1 Ra6 Rb1 Ra7+ Kf6 Ra5 Rb2 Ra6+ Kf7 Kh3 Rb1) 0.07/22 13} Ra8 {(Ra8 Kh3 Ke7 Kh4 Rg8 Ra6 Bd6 Bd4 Kd7 Kh3 Rg4 Kg2) 0.56/18 19} 51. Kg1 {(Kg1 Ra2 Kh2 Rb2 Kg2 Kg7 Ra6 Kf7 Ra7+ Kg6 Kh2) 0.07/21 13} Ke7 {(Ke7) 0.66/17 15} 52. Rc5 {(Rc5 Ke6 Kg2 Ra2 Rb5 Rb2 Ra5 Kf6 Ra6+) 0.07/22 13} Kd6 {(Kd6 Kg2 Rg8 Ra5 f4 Ra6+ Kd5 Ra5+ Ke6 Rxe5+ Kxe5 Bxf4+) 0.51/19 9} 53. Rb5 {(Rb5 Ra1+ Kg2 Ke6 Bc5 Ra2 Be3 Rb2) 0.07/22 13} Ra3 {(Rg8 Rb6+ Kd5 Rb5+ Ke6 Rb6+ Ke7 Rb5 Kf6 Rb6+ Kf7 Rb5) 0.07/18 29} 54. Kh2 {(Kh2 Ra1 Kg2 Ke6) 0.07/23 13} Ra6 {(Ra6 Kh3 Ra8 Rb6+ Kd5 Rb5+ Ke6 Rb6+ Kf7 Rb7+ Kf6 Rb6+) 0.10/19 11} 55. Kh3 {(Kh3 Ke6 Kg2 Ra2 Rb6+ Kd5 Rb7 Ke6 Rb5 Rb2) 0.07/22 13} Ke6 {(Ke6) 0.01/19 16} 56. Kg2 {(Kg2 Ra2 Rb6+ Kd5 Rb7 Ke6 Rb5 Bc3 Rb6+ Kf7 Rb7+ Kg6 Rb8 Rb2 Rg8+ Kf7 Rc8 Be5 Rc6 Kg7) 0.07/23 13} Bd6 {(Bd6) 0.26/18 10} 57. Rb7 {(Rb7 Be5 Kh2 Ra2 Rb6+ Kf7 Kg2 Bc3 Rb7+ Kg6 Rb8 Rb2 Rg8+ Kf7 Rc8 Be5 Rc6 Rb1 Ra6 Rb2 Ra7+) 0.07/23 12} Ra5 {(Ra5) 0.35/19 9} 58. Rb2 {(Rb2 Bc5 Bd2 Ra1 Rc2 Kd5 Bh6 Ra4 Bg5 Ra1 Bc1 Ra4) 0.03/18 12} Rd5 {(Rd5 Rb6 Ke7 Kh3 Rd3 Kh4 Rd1 Kh3 Kf7 g4 f4 Rxd6) 0.51/18 14} 59. Rb6 {(Rb6 Ke7 Rb1 Ke6 Rh1 Be5 Rh7 Rd1 Rh6+ Kf7 Ra6 Rb1) 0.07/22 12} Ke7 {(Ke7 Rb7+ Kf6 Rb6 Kf7 Rb7+ Kg6 Rb6 Kh5 f3 Rd3 fxe4) 0.17/19 10} 60. Rb1 {(Rb1 Ke6) 0.07/21 12} Rd3 {(Rd3 Kh3 Kf7 Kh4 Be7+ Kh3 Kg6 Rb6+ Rd6 Rb4 Rc6 Rb7) 0.32/18 10} 61. Ra1 {(Ra1 Be5 Ra7+ Kf6 Ra6+ Kf7 Kh2 Rd1 Kg2 Rb1) 0.07/20 12} Ke6 {(Ke6 Rb1 Be5 Rb6+ Rd6 Rxd6+ Bxd6 Kh3 Kf6 Bf4 Bc5 Be3) 0.40/17 8} 62. Ra6 {(Ra6 Kf7 Ra7+ Ke6 Rb7 Kd5) 0.07/21 12} Kf7 {(Kf7 Ra7+ Kg6 Ra6 Kg7 Ra7+ Kf6 Ra6 Rd5 f3 Ke6 Bf4) 0.07/20 9} 63. Ra7+ {(Ra7+ Ke6 Rb7 Rd5 Bg5 Ra5 Rb6 Kd5 Bd2 Ra2 Be3 Be5 Rb7) 0.07/22 12} Kf6 {(Kf6) 0.00/23 8} 64. Rb7 {(Rb7 Rd1 Kh2 Be5 Rb6+ Kf7 Rb5 Kf6 Ra5 Bd6 Kh3 Rd3 Kg2 Be5 Ra6+ Kf7) 0.07/22 12} Be5 {(Be5) 0.12/19 18} 65. Rb6+ {(Rb6+ Kf7 Rc6 Rd1 Ra6) 0.07/18 12} Ke7 {(Ke7) 0.12/19 8} 66. Bg5+ {(Bg5+ Kf7 Rb5 Ke6 Be3 Kf6 Rb4 Kg6 Rb6+ Kf7 Rc6) 0.07/18 12} Kd7 {(Kf7 Be3 Rc3 g4 fxg4 Rb4 Ke6 Rxe4 Kf5 Rb4 Rc2 Bd4) 0.38/18 8} 67. Ra6 {(Ra6 Bd6 Be3 Ke6 Rb6 Kf7 Rb7+) 0.07/20 12} Rf3 {(Rf3) 0.72/18 10} 68. Bf6 {(Bf6 Bd6 Ra5 Ke6 Bd4 Rd3 Be3 Be5 Ra6+ Kf7) 0.07/21 11} Bc7 {(Bc7 Bd4 Bd6 Ra4 Kc6 Ra5 Bc7 Rc5+ Kd6 Rc2 Kd5 Rxc7) 0.50/18 10} 69. Bg7 {(Bg7 Bd6 Ra5 Ke6 Bd4 Rd3) 0.07/19 11} Bb8 {(Bb8) 0.50/18 10} 70. Rb6 {(Rb6 Bd6) 0.07/19 11} Bd6 {(Bd6) 0.70/20 9} 71. Bd4 {(Bd4 Rd3 Be3 Ke6) 0.07/18 11} Ra3 {(Ra3 Be3 Ra4 Rb7+ Ke6 Ra7 Rxa7 Bxa7 Ke5 f3 Kd5 Kf2) 0.40/19 10} 72. Be3 {(Be3 Rd3 Rb5 Ke6 Bg5 Rd1 Rb6 Kf7 Ra6 Be5 Ra7+ Kg6 Be3 Rd3 Ra6+ Kf7 Ra7+) 0.07/22 11} Ra2 {(Ra2) 0.50/18 12} 73. Kh3 {(Kh3 Ra5 Kh2 Ra3 Rb7+ Ke6 Rb2 Kd5 Rb5+ Ke6 Rb1 Be5 Rb6+ Kf7 Rb7+ Ke6 Kg2 Ra2 Rb6+ Kd5) 0.07/18 11} Ra5 {(Ra5 Kh4 Rd5 g4 f4 Rxd6+ Kxd6 Bxf4+ Ke6 Be3 Ke5 Kg5) 0.31/18 12} 74. Rb2 {(Rb2 Ke6 Rc2 Ra1 Kg2 Rb1 Rc6 Kd7 Ra6 Be5 Ra7+ Ke6 Kh2 Kd5 Ra5+ Ke6 Kg2 Kf6 Ra6+ Kf7 Ra7+ Kf6) 0.07/21 11} Rd5 {(Rd5 Rc2 Rd3 Kh4 Ke6 Bf4 Bxf4 gxf4 Kd6 Kg5 Rd5 Kf6) 0.44/17 9} 75. Ra2 {(Ra2 Ke6 Ra6 Kf7 Ra7+ Kg6 Kg2) 0.07/16 11} Ke7 {(Ke7 Ra4 Kf6 Bd4+ Kg5 Rc4 Be7 Be3+ Kh5 g4+ fxg4+ Kg3) 0.42/17 10} 76. Ra7+ {(Ra7+ Kf6 Ra6 Kf7 Ra1) 0.07/15 11} Kf6 {(Kf6 Ra6 Kf7 Kh4 Be7+ Bg5 Bc5 Be3 Bxe3 fxe3 Rd3 Kg5) 0.00/20 21} 77. Ra6 {(Ra6 Kf7 Ra1 Rd3 Ra7+ Kf6 Kg2 Kg6 Ra1 Be5 Ra6+ Kf7 Ra7+) 0.07/22 11} Ke6 {(Ke6) 0.00/20 7} 78. Bf4 {(Bf4 Ke7 Bg5+ Kf7 Kg2 Be5 Ra7+ Kg6 Be3 Rd3 Ra6+ Kf7) 0.07/21 11} Rd3 {(Rd3) 0.00/22 10} 79. Kh4 {(Kh4 Ke7 Bg5+ Ke8 Ra8+ Kf7 Be3 Rd5 Kh3 Rd1 Kh2 Be5 Ra7+ Kf6 Ra6+ Kf7 Kg2) 0.07/19 11} Ke7 {(Ke7) 0.00/21 8} 80. Bg5+ {(Bg5+ Ke8 Ra8+ Kf7) 0.07/18 11} Kd7 {(Kd7 Be3 Rd5 Ra4 Be7+ Kh3 Rd3 Kg2 Ke6 g4 Bd6 Kf1) 0.09/17 7}"#;

        let pgn_info = get_pgn_info(sample_pgn).unwrap();

        assert_eq!(pgn_info.white_player, "RookieMonster 1.9.9 64-bit");
        assert_eq!(pgn_info.black_player, "Betsabe_II 2023");
        assert_eq!(pgn_info.date, "2025.01.06");
        assert!(pgn_info.out_of_book())
    }

    #[test]
    fn test_pgn_parsing_in_book_returns_true() {
        let sample_pgn = r#"[Site "114th Amateur D11"]
[Date "2025.01.06"]
[White "RookieMonster 1.9.9 64-bit"]
[Black "Betsabe_II 2023"]

1. d4 {(Book)} Nf6 {(Book)} 2. c4 {(Book)}"#;

        let pgn_info = get_pgn_info(sample_pgn).unwrap();
        assert!(!pgn_info.out_of_book())
    }
}