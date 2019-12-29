extern crate rand;

const WIDTH: usize = 4;
const NUM_BLOCKS: usize = WIDTH * WIDTH;

enum Direction {
    Down,
    Up,
    Left,
    Right,
}

type Section = [i32; WIDTH];
type Board = [i32; WIDTH * WIDTH];
const ZeroSection: Section = [0; WIDTH];

struct ThreesGame {
    blocks: Board,
    rng: rand::rngs::ThreadRng,
}

fn shift(in_sec: &Section, increasing: bool) -> Section {
    let mut oriented_sec: Section = *in_sec;
    if increasing {
        oriented_sec.reverse();
    }
    let mut out_sec = shift_down(&oriented_sec);
    if increasing {
        out_sec.reverse();
    }
    out_sec
}

fn shift_down(in_sec: &Section) -> Section {
    let mut out_sec = ZeroSection;

    let mut moved = false;
    let mut out_x = 0;
    for in_x in 0..WIDTH {
        if in_sec[in_x] == 0 && !moved {
            moved = true;
        } else {
            out_sec[out_x] = in_sec[in_x];
            out_x += 1;
        }
    }
    out_sec
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
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join("|")
            })
            .collect::<Vec<String>>()
            .join("\n")
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

    fn update(&mut self, d: Direction) -> bool {
        let increasing = match d {
            Direction::Up | Direction::Right => true,
            _ => false,
        };

        match d {
            Direction::Down | Direction::Up => {
                for i in 0..WIDTH {
                    let section = self.get_col(i);
                    self.set_col(i, shift(&section, increasing));
                }
            }
            Direction::Left | Direction::Right => {
                for i in 0..WIDTH {
                    let section = self.get_row(i);
                    self.set_row(i, shift(&section, increasing));
                }
            }
        };
        // check if this will be allowed
        if false {
            return false;
        }

        // We've shifted everything, we can add new elements now
        match d {
            Direction::Down | Direction::Up => {}
            Direction::Left | Direction::Right => {}
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift() {
        assert_eq!(shift(&[0, 0, 0, 0], false), [0, 0, 0, 0]);
        assert_eq!(shift(&[1, 0, 0, 0], false), [1, 0, 0, 0]);
        assert_eq!(shift(&[0, 1, 0, 0], false), [1, 0, 0, 0]);
        assert_eq!(shift(&[1, 1, 0, 0], false), [1, 1, 0, 0]);
        assert_eq!(shift(&[1, 0, 0, 1], false), [1, 0, 1, 0]);
        assert_eq!(shift(&[1, 1, 1, 1], false), [1, 1, 1, 1]);
        assert_eq!(shift(&[0, 0, 0, 0], true), [0, 0, 0, 0]);
        assert_eq!(shift(&[0, 0, 0, 1], true), [0, 0, 0, 1]);
        assert_eq!(shift(&[0, 0, 1, 0], true), [0, 0, 0, 1]);
        assert_eq!(shift(&[0, 0, 1, 1], true), [0, 0, 1, 1]);
        assert_eq!(shift(&[1, 0, 0, 1], true), [0, 1, 0, 1]);
        assert_eq!(shift(&[1, 1, 1, 1], true), [1, 1, 1, 1]);
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
}
