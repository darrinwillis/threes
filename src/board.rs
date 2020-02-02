pub const WIDTH: usize = 4;
const NUM_BLOCKS: usize = WIDTH * WIDTH;

#[derive(PartialEq, Copy, Clone, Debug, Enum)]
pub enum Direction {
    Down,
    Up,
    Left,
    Right,
}

pub type Rank = i32;
type BoardBlocks = [Rank; NUM_BLOCKS];
type Section = [Rank; WIDTH];
pub type BoardSections = [Section; WIDTH];
const ZERO_BOARD_BLOCKS: BoardBlocks = [0; NUM_BLOCKS];
const ZERO_SECTION: Section = [0; WIDTH];
const ZERO_BOARD_SECTIONS: BoardSections = [ZERO_SECTION; WIDTH];

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

#[derive(Copy, Clone, Debug)]
pub struct Board {
    blocks: BoardBlocks,
}

impl Board {
    pub fn new() -> Board {
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

    pub fn rows(&self) -> BoardSections {
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

    pub fn cols(&self) -> BoardSections {
        let mut sections = ZERO_BOARD_SECTIONS;
        for i in 0..WIDTH {
            sections[i] = self.get_col(i);
        }
        sections
    }

    pub fn values(&self) -> BoardBlocks {
        self.blocks
    }

    pub fn set_value(&mut self, row: usize, col: usize, value: Rank) {
        self.blocks[row * WIDTH + col] = value;
    }

    // Push the board in a certain direction
    // Returns true if the board was modified
    pub fn shove(&mut self, d: Direction) -> bool {
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
