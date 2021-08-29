use std::fs;

use super::agent_trainer::TrainingOutcomes;
use super::game::{Game, GameLog, MoveResult};

use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn read_game(result_file: &str, gen_id: i32) -> GameLog {
    let contents = fs::read_to_string(result_file).unwrap();
    let outcomes: TrainingOutcomes = serde_json::from_str(&contents).unwrap();
    outcomes
        .games_played
        .into_iter()
        .find(|g| g.gen_id == gen_id)
        .unwrap()
        .game_log
        .unwrap()
}

pub fn replay_game(result_file: &str, gen_id: i32) {
    let log = read_game(result_file, gen_id);
    interactive_step_game(log);
}

fn interactive_step_game(log: GameLog) {
    let do_logging = false;
    let mut game = Game::new(Some(log.seed), do_logging);
    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();
    write!(
        stdout,
        "{}{}q to exit. Any key to go to next move.{}\r\n",
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
    let mut move_index = 0;
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

        let advance = match c.unwrap() {
            Key::Char('q') => break,
            Key::Char('a') => true,
            _ => false,
        };

        let game_over = if advance {
            let direction = log.moves[move_index];
            move_index += 1;
            let move_result = game.update(direction);
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
        } else {
            false
        };
        if game_over {
            break;
        }
        println!("{}\n", game.render());
        stdout.flush().unwrap();
    }
}
