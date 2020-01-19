#[macro_use]
extern crate enum_map;
extern crate rand;
extern crate termion;

use enum_map::EnumMap;
use rand::prelude::*;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

const WIDTH: usize = 4;
const NUM_BLOCKS: usize = WIDTH * WIDTH;

#[derive(PartialEq, Copy, Clone, Enum)]
enum Direction {
    Down,
    Up,
    Left,
    Right,
}

type Score = i32;
type Rank = i32;
type BoardBlocks = [Rank; NUM_BLOCKS];
type Section = [Rank; WIDTH];
type BoardSections = [Section; WIDTH];
const ZERO_BOARD_BLOCKS: BoardBlocks = [0; NUM_BLOCKS];
const ZERO_SECTION: Section = [0; WIDTH];
const ZERO_BOARD_SECTIONS: BoardSections = [ZERO_SECTION; WIDTH];

#[derive(Copy, Clone)]
struct Board {
    blocks: BoardBlocks,
}

struct ThreesGame {
    cur_board: Board,
    shifted_boards: EnumMap<Direction, Option<Board>>,
    rng: rand::rngs::ThreadRng,
    // whether the board is empty; could be moved down into Board, but it's more efficient here.
    empty: bool,
}

struct GameResult {
    score: Score,
}

enum MoveResult {
    Moved(Option<GameResult>),
    Failed,
}

fn combine(in1: Rank, in2: Rank) -> Option<Rank> {
    if in1 == 0 || in2 == 0 {
        None
    } else if (in1 == 1 && in2 == 2) || (in1 == 2 && in2 == 1) {
        Some(3)
    } else if in1 == in2 && ![1, 2].contains(&in1) {
        Some(2 * in1)
    } else {
        None
    }
}

// Returns the new section and whether there was a shift in the section
fn shift(in_sec: &Section, increasing: bool) -> (Section, bool) {
    let mut oriented_sec: Section = *in_sec;
    if increasing {
        oriented_sec.reverse();
    }
    let (mut out_sec, moved) = shift_down(&oriented_sec);
    if increasing {
        out_sec.reverse();
    }
    (out_sec, moved)
}

// Returns the new section and whether there was a shift in the section
fn shift_down(in_sec: &Section) -> (Section, bool) {
    let mut out_sec = ZERO_SECTION;

    // If we have a movement, then all following blocks will shift down and
    // will not combine
    let mut will_shift = false;
    // We want to track if we had actual movement to report to the caller
    let mut block_moved = false;
    for in_x in 0..WIDTH - 1 {
        let bot = in_sec[in_x];
        let top = in_sec[in_x + 1];
        if bot == 0 || will_shift {
            out_sec[in_x] = top;
            will_shift = true;
            if top != 0 {
                block_moved = true;
            }
        } else if let Some(new_val) = combine(bot, top) {
            out_sec[in_x] = new_val;
            will_shift = true;
            block_moved = true;
        } else {
            out_sec[in_x] = bot;
            // XXX this is a little weird because this might get overwritten
            out_sec[in_x + 1] = top;
        }
    }
    (out_sec, block_moved)
}

impl Board {
    fn new() -> Board {
        Board {
            blocks: ZERO_BOARD_BLOCKS,
        }
    }

    fn get_row(&self, r: usize) -> Section {
        let mut new_row = ZERO_SECTION;
        for i in 0..WIDTH {
            new_row[i] = self.blocks[r * WIDTH + i];
        }
        new_row
    }

    fn set_row(&mut self, r: usize, sec: Section) {
        for i in 0..WIDTH {
            self.blocks[r * WIDTH + i] = sec[i];
        }
    }

    fn rows(&self) -> BoardSections {
        let mut sections = ZERO_BOARD_SECTIONS;
        for i in 0..WIDTH {
            sections[i] = self.get_row(i);
        }
        sections
    }

    fn get_col(&self, c: usize) -> Section {
        let mut new_col = ZERO_SECTION;
        for i in 0..WIDTH {
            new_col[i] = self.blocks[i * WIDTH + c];
        }
        new_col
    }

    fn set_col(&mut self, c: usize, sec: Section) {
        for i in 0..WIDTH {
            self.blocks[i * WIDTH + c] = sec[i];
        }
    }

    fn cols(&self) -> BoardSections {
        let mut sections = ZERO_BOARD_SECTIONS;
        for i in 0..WIDTH {
            sections[i] = self.get_col(i);
        }
        sections
    }

    fn values(&self) -> BoardBlocks {
        self.blocks
    }

    fn set_value(&mut self, row: usize, col: usize, value: Rank) {
        self.blocks[row * WIDTH + col] = value;
    }

    // Push the board in a certain direction
    // Returns true if the board was modified
    fn shove(&mut self, d: Direction) -> bool {
        let increasing = match d {
            Direction::Down | Direction::Right => true,
            _ => false,
        };

        let mut modified = false;
        match d {
            Direction::Down | Direction::Up => {
                for i in 0..WIDTH {
                    let section = self.get_col(i);
                    let (new_col, col_modified) = shift(&section, increasing);
                    if col_modified {
                        modified = true;
                        self.set_col(i, new_col);
                    }
                }
            }
            Direction::Left | Direction::Right => {
                for i in 0..WIDTH {
                    let section = self.get_row(i);
                    let (new_row, row_modified) = shift(&section, increasing);
                    if row_modified {
                        modified = true;
                        self.set_row(i, new_row);
                    }
                }
            }
        };
        modified
    }
}

impl ThreesGame {
    fn new() -> ThreesGame {
        ThreesGame {
            cur_board: Board::new(),
            shifted_boards: EnumMap::new(),
            empty: true,
            rng: rand::thread_rng(),
        }
    }

    // Returns the indexes of all the sections which are available
    fn elligible_sections(sections: BoardSections, sec_idx: usize) -> Vec<usize> {
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
            .blocks
            .chunks(WIDTH)
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
            Direction::Down,
            Direction::Up,
            Direction::Left,
            Direction::Right,
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

    fn update(&mut self, d: Direction) -> MoveResult {
        let modified = self.cur_board.shove(d);
        if !(modified || self.empty) {
            // Either this board was just modified by the shove or it's the first move
            MoveResult::Failed
        } else {
            // We've shifted everything, we can add new elements now
            let (new_row, new_col) = match d {
                Direction::Down | Direction::Up => {
                    let open_row = if d == Direction::Down { 0 } else { WIDTH - 1 };
                    let elligible_cols = Self::elligible_sections(self.cur_board.cols(), open_row);
                    if elligible_cols.is_empty() {
                        panic!("shifted board does not have open row")
                    }
                    let selected_idx = self.rng.gen_range(0, elligible_cols.len());
                    let selected_col = elligible_cols[selected_idx];
                    (open_row, selected_col)
                }
                Direction::Left | Direction::Right => {
                    let open_col = if d == Direction::Left { WIDTH - 1 } else { 0 };
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
                Direction::Down,
                Direction::Up,
                Direction::Left,
                Direction::Right,
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
    board.update(Direction::Left);
    println!("{}\n", board.render());
    board.update(Direction::Left);
    println!("{}\n", board.render());
    board.update(Direction::Left);
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
            Key::Left => Some(Direction::Left),
            Key::Right => Some(Direction::Right),
            Key::Up => Some(Direction::Up),
            Key::Down => Some(Direction::Down),
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
