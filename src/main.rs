extern crate termion;
extern crate rand;

use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;
use std::io::{Write, stdout, stdin};
use rand::prelude::*;

const WIDTH: usize = 4;
const NUM_BLOCKS: usize = WIDTH * WIDTH;

#[derive(PartialEq)]
enum Direction {
    Down,
    Up,
    Left,
    Right,
}

type Rank = i32;
type Section = [Rank; WIDTH];
type Board = [Rank; WIDTH * WIDTH];
const ZERO_SECTION: Section = [0; WIDTH];

struct ThreesGame {
    blocks: Board,
    rng: rand::rngs::ThreadRng,
}

fn combine(in1: Rank, in2: Rank) -> Option<Rank> {
    if in1 == 0 || in2 == 0 {
        None
    } else if (in1 == 1 && in2 == 2) || (in1 == 2 && in2 == 1) {
        Some(3)
    } else if in1 == in2 && ![1,2].contains(&in1) {
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
    for in_x in 0..WIDTH-1 {
        let bot = in_sec[in_x];
        let top = in_sec[in_x+1];
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
            out_sec[in_x+1] = top;
        }
    }
    (out_sec, block_moved)
}

impl ThreesGame {
    fn new() -> ThreesGame {
        ThreesGame {
            blocks: [0; NUM_BLOCKS],
            rng: rand::thread_rng(),
        }
    }

    fn render(&self) -> String {
        self.blocks
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

    fn get_row(&self, r: usize) -> Section {
        let mut new_row: [i32; WIDTH] = [0; WIDTH];
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

    fn rows(&self) -> Vec<Section> {
        (0..WIDTH).map(|i| self.get_row(i)).collect()
    }

    fn get_col(&self, c: usize) -> Section {
        let mut new_col: [i32; WIDTH] = [0; WIDTH];
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

    fn cols(&self) -> Vec<Section> {
        (0..WIDTH).map(|i| self.get_col(i)).collect()
    }

    fn set_value(&mut self, row: usize, col: usize, value: Rank) {
        self.blocks[row * WIDTH + col] = value;
    }

    fn update(&mut self, d: Direction) -> bool {
        let increasing = match d {
            Direction::Down | Direction::Right => true,
            _ => false,
        };

        match d {
            Direction::Down | Direction::Up => {
                for i in 0..WIDTH {
                    let section = self.get_col(i);
                    let (new_col, _) = shift(&section, increasing);
                    self.set_col(i, new_col);
                }
            }
            Direction::Left | Direction::Right => {
                for i in 0..WIDTH {
                    let section = self.get_row(i);
                    let (new_row, _) = shift(&section, increasing);
                    self.set_row(i, new_row);
                }
            }
        };

        // We've shifted everything, we can add new elements now
        let new_pos = match d {
            Direction::Down | Direction::Up => {
                let open_row = if d == Direction::Down {0} else {WIDTH-1};
                let elligible_cols = self.cols().iter().enumerate()
                    .filter_map(|(col_idx, col)| if col[open_row] == 0 {Some(col_idx)} else {None})
                    .collect::<Vec<usize>>();
                if elligible_cols.len() > 0{
                    let selected_idx = self.rng.gen_range(0, elligible_cols.len());
                    let selected_col = elligible_cols[selected_idx];
                    Some((open_row, selected_col))
                } else {
                    None
                }
            }
            Direction::Left | Direction::Right => {
                let open_col = if d == Direction::Left {WIDTH-1} else {0};
                let elligible_rows = self.rows().iter().enumerate()
                    .filter_map(|(row_idx, row)| if row[open_col] == 0 {Some(row_idx)} else {None})
                    .collect::<Vec<usize>>();
                if elligible_rows.len() > 0 {
                    let selected_idx = self.rng.gen_range(0, elligible_rows.len());
                    let selected_row = elligible_rows[selected_idx];
                    Some((selected_row, open_col))
                } else {
                    None
                }
            }
        };
        match new_pos {
            Some((new_row, new_col)) => {
                let new_val = self.rng.gen_range(1,2 + 1);
                self.set_value(new_row, new_col, new_val);
                true
            }
            None => false
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
    write!(stdout, "{}{}q to exit. Arrows keys to play.{}\r\n",
           // Clear the screen.
           termion::clear::All,
           // Goto (1,1).
           termion::cursor::Goto(1, 1),
           // Hide the cursor.
           termion::cursor::Hide).unwrap();
    println!("{}\n", board.render());

    let mut first = true;
    for c in stdin.keys() {
        if first {
            write!(stdout, "{}", termion::clear::All).unwrap();
            first = false;
        }
        write!(stdout, "{}{}", termion::cursor::Goto(1, 1), termion::clear::CurrentLine).unwrap();

        let direction = match c.unwrap() {
            Key::Char('q') => break,
            Key::Left => Some(Direction::Left),
            Key::Right => Some(Direction::Right),
            Key::Up => Some(Direction::Up),
            Key::Down => Some(Direction::Down),
            _ => None
        };
        match direction {
            Some(d) => board.update(d),
            None => false
        };
        println!("{}\n", board.render());
        stdout.flush().unwrap();

    }
}
