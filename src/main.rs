#[macro_use]
extern crate enum_map;
extern crate clap;
extern crate histogram;
#[macro_use]
extern crate itertools;
extern crate pprof;
extern crate rand;
extern crate termion;

mod agent_runner;
mod agent_trainer;
mod board;
mod game;
mod q_agent;
mod random_agent;
mod utils;

use clap::{App, Arg, SubCommand};
use enum_map::EnumMap;
use rand::prelude::*;
use rayon::prelude::*;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::time::Instant;
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

fn play_games(num_games: usize, seed: Option<&mut StdRng>) -> Vec<(StdRng, game::GameResult)> {
    let mut rng = utils::resolve_rng_from_seed(seed);
    // Play some games
    let game_rngs = (0..num_games)
        .map(|_i| StdRng::from_rng(&mut rng).unwrap())
        .collect::<Vec<StdRng>>();
    game_rngs
        .into_par_iter()
        .map(|seed_rng| {
            // This must be first; we want to stash the original rng away for later
            let mut game_rng = seed_rng.clone();
            let agent = &mut random_agent::RandomAgent::new(Some(&mut game_rng));
            let result = agent_runner::play_game(Some(&mut game_rng), agent);
            (seed_rng, result)
        })
        .collect()
}

fn play_and_analyze_games(num_games: usize) {
    let start = Instant::now();
    let results = play_games(num_games, None);
    let end = Instant::now();
    let duration = end - start;
    let scores = results
        .iter()
        .map(|r| r.1.score)
        .collect::<Vec<game::Score>>();
    let mut histogram = histogram::Histogram::new();
    for s in scores {
        histogram.increment(s as u64).unwrap();
    }
    let (_best_seed, best_result) = results.into_iter().max_by_key(|r| r.1.score).unwrap();
    println!(
        "Played {} random games in {}s ({}games/s). Max Score: {}",
        num_games,
        duration.as_secs_f32(),
        num_games as f32 / duration.as_secs_f32(),
        best_result.score,
    );
    println!("  {:5}: {}", "min", histogram.minimum().unwrap());
    for p in vec![1.0, 10.0, 50.0, 90.0, 99.0, 99.9] {
        println!(" p{:5}: {}", p, histogram.percentile(p).unwrap());
    }
    println!("  {:5}: {}", "max", histogram.maximum().unwrap());
    println!(" {:5}: {}", "stddev", histogram.stddev().unwrap());
    let best_board = best_result.final_render;
    println!("winning board\n{}", best_board);
}

fn train_q_agent() {
    let filename = "train_results.json";

    let mut agent = q_agent::QAgent::new(None);
    let train_result = agent_trainer::train_agent_from_scratch(&mut agent);

    let mut file = File::create(filename).unwrap();
    let contents = serde_json::to_string(&train_result.outcomes).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
}

fn main() {
    let matches = App::new("Threes Engine")
        .version("0.1")
        .author("Darrin Willis <willisdarrin@gmail.com")
        .about("threes engine")
        .arg(
            Arg::with_name("profile")
                .short("p")
                .help("generate profiling flamegraph"),
        )
        .subcommand(SubCommand::with_name("interactive").about("play a game as a human"))
        .subcommand(SubCommand::with_name("random").about("random agent to play a game"))
        .subcommand(SubCommand::with_name("train").about("train a q agent to play"))
        .get_matches();

    let profile = matches.is_present("profile");
    let guard = if profile {
        Some(pprof::ProfilerGuard::new(100).unwrap())
    } else {
        None
    };
    if matches.is_present("interactive") {
        play_interactive_game()
    } else if matches.is_present("random") {
        let num_games = 100_000;
        play_and_analyze_games(num_games)
    } else if matches.is_present("train") {
        train_q_agent()
    }
    if let Some(g) = guard {
        if let Ok(report) = g.report().build() {
            println!("report: {:?}", &report);
            let file = File::create("flamegraph.svg").unwrap();
            report.flamegraph(file).unwrap();
        };
    }
}
