#[macro_use]
extern crate enum_map;
extern crate rand;
extern crate termion;

mod board;
mod game;
mod random_player;

use enum_map::EnumMap;
use std::io::{stdin, stdout, Write};
use std::time::{Duration, Instant};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn play_interactive_game() {
    let mut game = game::Game::new(None);
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
    println!("{}\n", game.render());

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
                let move_result = game.update(d);
                match move_result {
                    game::MoveResult::Moved(Some(game_result)) => {
                        write!(
                            stdout,
                            "{}Game Over\r\n{} points\r\n",
                            termion::clear::All,
                            game_result.score
                        )
                        .unwrap();
                        true
                    }
                    game::MoveResult::Moved(None) => {
                        // The game is still going
                        false
                    }
                    game::MoveResult::Failed => false,
                }
            }
            // Unknown move direction
            None => false,
        };
        if game_over {
            break;
        }
        println!("{}\n", game.render());
        stdout.flush().unwrap();
    }
}

fn main() {
    println!("Hello, world!");
    let interactive = false;
    if interactive {
        play_interactive_game();
    } else {
        let mut results = Vec::new();
        let num_games = 100000;

        let start = Instant::now();
        // Play some games
        for _ in 0..num_games {
            results.push(random_player::play_game());
        }
        let end = Instant::now();
        let duration = end - start;
        let max_score = results.iter().map(|r| r.score).max().unwrap();
        println!(
            "Played {} random games in {}s ({}games/s). Max Score: {}",
            results.len(),
            duration.as_secs_f32(),
            results.len() as f32 / duration.as_secs_f32(),
            max_score,
        );
    }
}
