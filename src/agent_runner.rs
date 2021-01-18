use rand::prelude::*;

use super::board;
use super::game;
use super::utils;

pub trait Agent {
    fn take_action(&mut self, game: &game::Game, train_mode: bool) -> board::Direction;
    fn update(
        &mut self,
        board: &board::Board,
        action: board::Direction,
        new_board: &board::Board,
        reward: f64,
    );
    fn print(&self);
}

pub fn play_game<A: Agent>(
    seed: Option<&mut StdRng>,
    agent: &mut A,
    train_mode: bool,
) -> game::GameResult {
    let mut rng = utils::resolve_rng_from_seed(seed);
    // Only log if we're testing
    let do_logging = !train_mode;
    let mut game = game::Game::new(Some(rng.next_u64()), do_logging);

    let first_direction = agent.take_action(&game, train_mode);
    game.update(first_direction);

    loop {
        let options = game.available_moves();
        assert!(!options.is_empty());
        let direction = agent.take_action(&game, train_mode);
        let prev_board = game.cur_board;
        let prev_score = game.cur_score();
        let move_result = game.update(direction);
        let new_score = game.cur_score();
        let reward = new_score - prev_score;
        if train_mode {
            agent.update(&prev_board, direction, &game.cur_board, reward as f64);
        }
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
