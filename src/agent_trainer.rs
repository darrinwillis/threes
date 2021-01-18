use serde::{Deserialize, Serialize};

use super::agent_runner;
use super::agent_runner::Agent;
use super::game::GameLog;
use super::game::Score;
use super::utils;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayedGame {
    pub gen_id: i32,
    pub score: Score,
    pub game_log: Option<GameLog>,
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
    let mut rng = utils::resolve_rng_from_seed(None);

    let mut games_played = Vec::new();
    for gen_id in 0..num_generations {
        // Train
        for _episode in 0..num_episodes_per_gen {
            // Note that we're running with the SAME game every time here
            let mut new_rng = utils::resolve_rng_from_seed(Some(&mut rng));
            let _result = agent_runner::play_game(Some(&mut new_rng), agent, true);
        }

        // Test
        let mut new_rng = utils::resolve_rng_from_seed(Some(&mut rng));
        let result = agent_runner::play_game(Some(&mut new_rng), agent, false);
        let score = result.score;
        let game_log = result.log;
        games_played.push(PlayedGame {
            gen_id,
            score,
            game_log,
        });
    }
    let outcomes = TrainingOutcomes { games_played };
    TrainResult { outcomes, agent }
}
