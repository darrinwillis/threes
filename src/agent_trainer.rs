use serde::{Deserialize, Serialize};

use super::agent_runner;
use super::agent_runner::Agent;
use super::utils;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayedGame {
    pub gen_id: i32,
    pub score: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrainingOutcomes {
    pub games_played: Vec<PlayedGame>,
}

pub struct TrainResult<'a, A: Agent> {
    pub outcomes: TrainingOutcomes,
    pub agent: &'a A,
}

pub fn train_agent_from_scratch<A: Agent>(
    agent: &mut A,
    num_generations: i32,
    num_episodes_per_gen: i32,
) -> TrainResult<A> {
    let rng = utils::resolve_rng_from_seed(None);

    let mut games_played = Vec::new();
    for gen_id in 0..num_generations {
        // Train
        for _episode in 0..num_episodes_per_gen {
            // Note that we're running with the SAME game every time here
            let _result = agent_runner::play_game(Some(&mut rng.clone()), agent, true);
        }

        // Test
        let result = agent_runner::play_game(Some(&mut rng.clone()), agent, false);
        let score = result.score;
        games_played.push(PlayedGame { gen_id, score });
    }
    let outcomes = TrainingOutcomes { games_played };
    TrainResult { outcomes, agent }
}
