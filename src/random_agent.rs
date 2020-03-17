use rand::prelude::*;

use super::agent_runner::Agent;
use super::board;
use super::game;
use super::utils;

pub struct RandomAgent {
    rng: StdRng,
}

impl RandomAgent {
    pub fn new(seed: Option<&mut StdRng>) -> RandomAgent {
        RandomAgent {
            rng: utils::resolve_rng_from_seed(seed),
        }
    }
}

impl Agent for RandomAgent {
    fn take_action(&mut self, game: &game::Game) -> board::Direction {
        let options = game.available_moves();
        assert!(!options.is_empty());
        let num_options = options.len();
        let op_idx = self.rng.gen_range(0, num_options);
        options[op_idx]
    }
}
