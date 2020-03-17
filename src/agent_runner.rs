use rand::prelude::*;

use super::board;
use super::game;

pub trait Agent {
    fn take_action(&mut self, game: &game::Game) -> board::Direction;
}

pub fn play_game<A: Agent>(seed: Option<&mut StdRng>, agent: &mut A) -> game::GameResult {
    let mut rng = match seed {
        None => StdRng::from_rng(rand::thread_rng()),
        Some(seed_rng) => StdRng::from_rng(seed_rng),
    }
    .unwrap();
    let mut game = game::Game::new(Some(&mut rng));

    let first_direction = agent.take_action(&game);
    game.update(first_direction);

    loop {
        let options = game.available_moves();
        assert!(!options.is_empty());
        let direction = agent.take_action(&game);
        let move_result = game.update(direction);
        // We already checked the available moves, this should work
        match move_result {
            game::MoveResult::Moved(Some(result)) => {
                assert!(game.available_moves().is_empty());
                return result;
            }
            game::MoveResult::Moved(None) => {}
            game::MoveResult::Failed => panic!("taking available move failed to update"),
        }
    }
}
