#[macro_use]
extern crate enum_map;
extern crate rand;
extern crate termion;

mod board;

use enum_map::EnumMap;
use rand::prelude::*;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct ThreesGame {
    cur_board: board::Board,
    shifted_boards: EnumMap<board::Direction, Option<board::Board>>,
    rng: rand::rngs::ThreadRng,
    // whether the board is empty; could be moved down into Board, but it's more efficient here.
    empty: bool,
}

type Score = i32;
struct GameResult {
    score: Score,
}

enum MoveResult {
    Moved(Option<GameResult>),
    Failed,
}

impl ThreesGame {
    fn new() -> ThreesGame {
        ThreesGame {
            cur_board: board::Board::new(),
            shifted_boards: EnumMap::new(),
            empty: true,
            rng: rand::thread_rng(),
        }
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

    fn render(&self) -> String {
        self.cur_board
            .rows()
            .iter()
            .map(|row| {
                row.iter()
                    .map(|c| format!("{:3}", c.to_string()))
                    .collect::<Vec<String>>()
                    .join("|")
            })
            .collect::<Vec<String>>()
            .join("\r\n")
    }

    fn check_game_over(&self) -> Option<GameResult> {
        let no_moves = vec![
            board::Direction::Down,
            board::Direction::Up,
            board::Direction::Left,
            board::Direction::Right,
        ]
        .iter()
        .all(|d| self.shifted_boards[*d].is_none());
        if !no_moves {
            return None;
        }

        Some(GameResult {
            score: self.cur_board.values().iter().map(|v| v * v).sum(),
        })
    }

    fn update(&mut self, d: board::Direction) -> MoveResult {
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
            let new_val = self.rng.gen_range(1, 2 + 1);
            self.cur_board.set_value(new_row, new_col, new_val);
            self.empty = false;
            for d in vec![
                board::Direction::Down,
                board::Direction::Up,
                board::Direction::Left,
                board::Direction::Right,
            ] {
                let mut new_board = self.cur_board;
                let modified = new_board.shove(d);
                self.shifted_boards[d] = if modified { Some(new_board) } else { None }
            }
            MoveResult::Moved(self.check_game_over())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift() {
        // Test simple shifting for forwards and backwards
        assert_eq!(shift(&[0, 0, 0, 0], false), ([0, 0, 0, 0], false));
        assert_eq!(shift(&[1, 0, 0, 0], false), ([1, 0, 0, 0], false));
        assert_eq!(shift(&[0, 1, 0, 0], false), ([1, 0, 0, 0], true));
        assert_eq!(shift(&[1, 1, 0, 0], false), ([1, 1, 0, 0], false));
        assert_eq!(shift(&[1, 0, 0, 1], false), ([1, 0, 1, 0], true));
        assert_eq!(shift(&[1, 1, 1, 1], false), ([1, 1, 1, 1], false));
        assert_eq!(shift(&[0, 0, 0, 0], true), ([0, 0, 0, 0], false));
        assert_eq!(shift(&[0, 0, 0, 1], true), ([0, 0, 0, 1], false));
        assert_eq!(shift(&[0, 0, 1, 0], true), ([0, 0, 0, 1], true));
        assert_eq!(shift(&[0, 0, 1, 1], true), ([0, 0, 1, 1], false));
        assert_eq!(shift(&[1, 0, 0, 1], true), ([0, 1, 0, 1], true));
        assert_eq!(shift(&[1, 1, 1, 1], true), ([1, 1, 1, 1], false));

        // Test combining
        assert_eq!(shift(&[1, 2, 0, 0], false), ([3, 0, 0, 0], true));
        assert_eq!(shift(&[2, 1, 0, 0], false), ([3, 0, 0, 0], true));
        assert_eq!(shift(&[3, 3, 0, 0], false), ([6, 0, 0, 0], true));
        assert_eq!(shift(&[3, 6, 0, 0], false), ([3, 6, 0, 0], false));
        assert_eq!(shift(&[6, 6, 0, 0], false), ([12, 0, 0, 0], true));
    }
}

fn main() {
    println!("Hello, world!");
    let mut board = ThreesGame::new();
    println!("Empty board:\n{}\n", board.render());
    board.update(board::Direction::Left);
    println!("{}\n", board.render());
    board.update(board::Direction::Left);
    println!("{}\n", board.render());
    board.update(board::Direction::Left);
    println!("{}\n", board.render());
    board = ThreesGame::new();

    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();
    write!(
        stdout,
        "{}{}q to exit. Arrows keys to play.{}\r\n",
        // Clear the screen.
        termion::clear::All,
        // Goto (1,1).
        termion::cursor::Goto(1, 1),
        // Hide the cursor.
        termion::cursor::Hide
    )
    .unwrap();
    println!("{}\n", board.render());

    let mut first = true;
    for c in stdin.keys() {
        if first {
            write!(stdout, "{}", termion::clear::All).unwrap();
            first = false;
        }
        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(1, 1),
            termion::clear::CurrentLine
        )
        .unwrap();

        let direction = match c.unwrap() {
            Key::Char('q') => break,
            Key::Left => Some(board::Direction::Left),
            Key::Right => Some(board::Direction::Right),
            Key::Up => Some(board::Direction::Up),
            Key::Down => Some(board::Direction::Down),
            _ => None,
        };
        let game_over = match direction {
            Some(d) => {
                let move_result = board.update(d);
                match move_result {
                    MoveResult::Moved(Some(game_result)) => {
                        write!(
                            stdout,
                            "{}Game Over\r\n{} points\r\n",
                            termion::clear::All,
                            game_result.score
                        )
                        .unwrap();
                        true
                    }
                    MoveResult::Moved(None) => {
                        // The game is still going
                        false
                    }
                    MoveResult::Failed => false,
                }
            }
            // Unknown move direction
            None => false,
        };
        if game_over {
            break;
        }
        println!("{}\n", board.render());
        stdout.flush().unwrap();
    }
}
