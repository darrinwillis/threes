extern crate termion;

use rand::prelude::*;

use super::board;
use super::utils;

pub type Score = i32;
pub struct GameResult {
    pub score: Score,
    pub num_moves: i32,
    pub final_board: board::Board,
    pub final_render: String,
}

pub enum MoveResult {
    Moved(Option<GameResult>),
    Failed,
}

pub struct Game {
    pub cur_board: board::Board,
    shifted_boards: crate::EnumMap<board::Direction, Option<board::Board>>,
    num_moves: i32,
    rng: rand::rngs::StdRng,
    next_rank: board::Rank,
    parity: i32, // count of 2s - count of 1s
    // whether the board is empty; could be moved down into Board, but it's more efficient here.
    empty: bool,
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
    pub fn new(seed: Option<&mut StdRng>) -> Game {
        let mut rng = utils::resolve_rng_from_seed(seed);
        let first_rank = Game::rand_rank(&mut rng);
        let new_board = board::Board::new();
        Game {
            cur_board: new_board,
            shifted_boards: Self::take_all_moves(&new_board),
            empty: true,
            num_moves: 0,
            rng,
            parity: 0,
            next_rank: first_rank,
        }
    }

    fn rand_rank(rng: &mut StdRng) -> board::Rank {
        rng.gen_range(1, 2 + 1)
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

    fn check_game_over(&self) -> Option<GameResult> {
        let no_moves = board::ALL_DIRECTIONS
            .iter()
            .all(|d| self.shifted_boards[*d].is_none());
        if !no_moves {
            return None;
        }

        Some(GameResult {
            score: self.cur_board.values().iter().map(|v| v * v).sum(),
            num_moves: self.num_moves,
            final_board: self.cur_board,
            final_render: self.render(),
        })
    }

    fn take_next_rank(&mut self) -> board::Rank {
        let ret = self.next_rank;
        let range = 100;
        let multiplier = 3;
        let threshold = 50 + multiplier * self.parity;
        self.next_rank = if self.rng.gen_range(0, range) > threshold {
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
            next_boards[*d] = if modified { Some(new_board) } else { None }
        }
        next_boards
    }

    pub fn available_moves(&self) -> Vec<board::Direction> {
        let next_moves = self
            .shifted_boards
            .iter()
            .filter_map(|(k, v)| if v.is_some() { Some(k) } else { None })
            .collect::<Vec<board::Direction>>();
        if next_moves.len() == 0 && self.cur_board.is_empty() {
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
                    let selected_idx = self.rng.gen_range(0, elligible_cols.len());
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
                    let selected_idx = self.rng.gen_range(0, elligible_rows.len());
                    let selected_row = elligible_rows[selected_idx];
                    (selected_row, open_col)
                }
            };
            let new_val = self.take_next_rank();
            self.cur_board.set_value(new_row, new_col, new_val);
            self.num_moves += 1;
            self.empty = false;
            self.shifted_boards = Self::take_all_moves(&self.cur_board);
            MoveResult::Moved(self.check_game_over())
        }
    }
}
