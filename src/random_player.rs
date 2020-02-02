use rand::prelude::*;

use super::board;
use super::game;

fn rand_direction(rng: &mut StdRng) -> board::Direction {
    match rng.gen_range(0, 4) {
        0 => board::Direction::Down,
        1 => board::Direction::Up,
        2 => board::Direction::Left,
        3 => board::Direction::Right,
        _ => panic!("rng returned unexpected value"),
    }
}

pub fn play_game(seed: Option<StdRng>) -> game::GameResult {
    let mut rng = match seed {
        None => StdRng::from_rng(rand::thread_rng()),
        Some(seed_rng) => StdRng::from_rng(seed_rng),
    }
    .unwrap();
    let mut game = game::Game::new(Some(&mut rng));

    let first_move = rand_direction(&mut rng);
    game.update(first_move);

    loop {
        let options = game.available_moves();
        assert!(!options.is_empty());
        let op_idx = rng.gen_range(0, options.len());
        let direction = options[op_idx];
        let move_result = game.update(direction);
        // We already checked the available moves, this should work
        match move_result {
            game::MoveResult::Moved(Some(result)) => {
                return result;
            }
            game::MoveResult::Moved(None) => {}
            game::MoveResult::Failed => panic!("taking available move failed to update"),
        }
    }
}
