extern crate termion;

use std::convert::TryInto;

//use rand::prelude::*;
use crate::rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoroshiro128StarStar;
use serde::{Deserialize, Serialize};

use super::board;

pub type Score = i64;

pub type RngType = Xoroshiro128StarStar;

#[derive(Serialize, Deserialize, Debug)]
pub struct GameResult {
    pub score: Score,
    pub num_moves: i32,
    pub final_board: board::Board,
    pub final_render: String,
    // Only present if game was played with logging on
    pub log: Option<GameLog>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameLog {
    pub seed: u64,
    pub moves: Vec<board::Direction>,
}

pub enum MoveResult {
    Moved(Option<GameResult>),
    Failed,
}

pub struct Game {
    //
    // Game state
    //
    pub cur_board: board::Board,
    next_rank: board::Rank,
    // (count of 2s - count of 1s) used to bias the RNG towards balanced 1s and 2s
    parity: i32,
    // The next state if the player chooses that direction
    shifted_boards: crate::EnumMap<board::Direction, Option<board::Board>>,
    // whether the board is empty; could be moved down into Board, but it's more efficient here.
    empty: bool,
    //
    // Game history
    //
    num_moves: i32,
    moves: Option<Vec<board::Direction>>,
    // Unique identifier for this game. Games are reproducible.
    pub seed: u64,
    rng: Xoroshiro128StarStar,
}

fn color_rank(rank: board::Rank) -> String {
    if rank == 1 {
        format!(
            "{}{:3}{}",
            termion::color::Fg(termion::color::Red),
            rank,
            termion::color::Fg(termion::color::Reset)
        )
    } else if rank == 2 {
        format!(
            "{}{:3}{}",
            termion::color::Fg(termion::color::Blue),
            rank,
            termion::color::Fg(termion::color::Reset)
        )
    } else if rank == 0 {
        format!("{:3}", "")
    } else {
        format!("{:3}", rank)
    }
}

impl Game {
    pub fn new(seed: Option<u64>, do_logging: bool) -> Game {
        let seed = seed.unwrap_or(0);
        let mut rng = RngType::seed_from_u64(seed);
        let first_rank = Game::rand_rank(&mut rng);
        let new_board = board::Board::new();
        Game {
            seed,
            cur_board: new_board,
            shifted_boards: Self::take_all_moves(&new_board),
            empty: true,
            num_moves: 0,
            moves: if do_logging { Some(Vec::new()) } else { None },
            rng,
            parity: 0,
            next_rank: first_rank,
        }
    }

    pub fn score(board: &board::Board) -> Score {
        fn score_tile(rank: board::Rank) -> i64 {
            let base: f64 = 3.0;
            let raw: f64 = rank.try_into().unwrap();
            return base.powf(((raw / 3.0).log2() + 1.0).try_into().unwrap()) as i64;
        }
        board.values().iter().map(|v| score_tile(*v)).sum()
    }

    fn rand_rank(rng: &mut RngType) -> board::Rank {
        rng.gen_range(1..=2)
    }

    // Returns the indexes of all the sections which are available
    fn elligible_sections(sections: board::BoardSections, sec_idx: usize) -> Vec<usize> {
        sections
            .iter()
            .enumerate()
            .filter_map(|(col_idx, col)| {
                if col[sec_idx] == 0 {
                    Some(col_idx)
                } else {
                    None
                }
            })
            .collect::<Vec<usize>>()
    }

    pub fn render(&self) -> String {
        let mut rows = self
            .cur_board
            .rows()
            .iter()
            .map(|row| {
                row.iter()
                    .map(|c| color_rank(*c))
                    .collect::<Vec<String>>()
                    .join("|")
            })
            .collect::<Vec<String>>();
        rows[0] = format!(
            "{}{:5}|{}| <- next up",
            rows[0],
            "",
            color_rank(self.next_rank)
        );
        rows.join("\r\n")
    }

    pub fn cur_score(&self) -> Score {
        return Game::score(&self.cur_board);
    }

    fn check_game_over(&self) -> Option<GameResult> {
        let no_moves = board::ALL_DIRECTIONS
            .iter()
            .all(|d| self.shifted_boards[*d].is_none());
        if !no_moves {
            return None;
        }

        Some(GameResult {
            score: self.cur_score(),
            num_moves: self.num_moves,
            final_board: self.cur_board,
            final_render: self.render(),
            log: self.moves.as_ref().map(|moves| GameLog {
                seed: self.seed,
                moves: moves.to_vec(),
            }),
        })
    }

    fn take_next_rank(&mut self) -> board::Rank {
        let ret = self.next_rank;
        let range = 100;
        let multiplier = 3;
        let threshold = 50 + multiplier * self.parity;
        self.next_rank = if self.rng.gen_range(0..range) > threshold {
            2
        } else {
            1
        };
        if ret == 2 {
            self.parity += 1;
        } else if ret == 1 {
            self.parity -= 1;
        }
        ret
    }

    fn take_all_moves(
        board: &board::Board,
    ) -> crate::EnumMap<board::Direction, Option<board::Board>> {
        let mut next_boards = crate::EnumMap::new();
        for d in &board::ALL_DIRECTIONS {
            let mut new_board = *board;
            let modified = new_board.shove(*d);
            next_boards[*d] = if modified || new_board.is_empty() {
                Some(new_board)
            } else {
                None
            }
        }
        next_boards
    }

    pub fn available_moves(&self) -> Vec<board::Direction> {
        let next_moves = self
            .shifted_boards
            .iter()
            .filter_map(|(k, v)| if v.is_some() { Some(k) } else { None })
            .collect::<Vec<board::Direction>>();
        if next_moves.is_empty() && self.cur_board.is_empty() {
            board::ALL_DIRECTIONS.to_vec()
        } else {
            next_moves
        }
    }

    pub fn update(&mut self, d: board::Direction) -> MoveResult {
        let modified = self.cur_board.shove(d);
        if !(modified || self.empty) {
            // Either this board was just modified by the shove or it's the first move
            MoveResult::Failed
        } else {
            // We've shifted everything, we can add new elements now
            let (new_row, new_col) = match d {
                board::Direction::Down | board::Direction::Up => {
                    let open_row = if d == board::Direction::Down {
                        0
                    } else {
                        board::WIDTH - 1
                    };
                    let elligible_cols = Self::elligible_sections(self.cur_board.cols(), open_row);
                    if elligible_cols.is_empty() {
                        panic!("shifted board does not have open row")
                    }
                    let selected_idx = self.rng.gen_range(0..elligible_cols.len());
                    let selected_col = elligible_cols[selected_idx];
                    (open_row, selected_col)
                }
                board::Direction::Left | board::Direction::Right => {
                    let open_col = if d == board::Direction::Left {
                        board::WIDTH - 1
                    } else {
                        0
                    };
                    let elligible_rows = Self::elligible_sections(self.cur_board.rows(), open_col);
                    if elligible_rows.is_empty() {
                        panic!("shifted board does not have open col")
                    }
                    let selected_idx = self.rng.gen_range(0..elligible_rows.len());
                    let selected_row = elligible_rows[selected_idx];
                    (selected_row, open_col)
                }
            };
            let new_val = self.take_next_rank();
            self.cur_board.set_value(new_row, new_col, new_val);
            self.num_moves += 1;
            self.moves.as_mut().map(|moves| moves.push(d));
            self.empty = false;
            self.shifted_boards = Self::take_all_moves(&self.cur_board);
            MoveResult::Moved(self.check_game_over())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_play() {
        for do_logging in vec![false, true] {
            let mut game = Game::new(None, do_logging);

            let mut i = 0;
            let mut moves_taken = vec![];
            let result = loop {
                println!("Board #{}:\n{}", i, game.render());
                if let Some(game_result) = game.check_game_over() {
                    break game_result;
                }

                let moves = game.available_moves();
                // We just checked for game over
                assert_ne!(moves.len(), 0);

                let first_move = moves[0];
                moves_taken.push(first_move);

                let prev_score = game.cur_score();

                let move_result = game.update(first_move);
                if let MoveResult::Failed = move_result {
                    // We shouldn't be able to take a failed move
                    assert!(false);
                }
                assert!(game.cur_score() >= prev_score);
                i += 1;
            };
            assert_ne!(result.num_moves, 0);
            assert_ne!(result.score, 0);
            let serialized = serde_json::to_string(&result).unwrap();
            println!("serialized result:\n{}", serialized);

            // Log tests
            assert_eq!(result.log.is_some(), do_logging);
            if do_logging {
                let log = result.log.unwrap();
                assert_eq!(log.moves.len() as i32, result.num_moves);
                assert_eq!(log.moves, moves_taken);
            }
        }
    }

    #[test]
    fn test_score() {
        let b_3 =
            board::Board::from_rows(&[[3, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let b_6 =
            board::Board::from_rows(&[[6, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let b_12 =
            board::Board::from_rows(&[[12, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);

        let b_33 =
            board::Board::from_rows(&[[3, 3, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        assert_eq!(Game::score(&b_3), 3);
        assert_eq!(Game::score(&b_6), 9);
        assert_eq!(Game::score(&b_12), 27);
        assert_eq!(Game::score(&b_33), 6);
    }
}
