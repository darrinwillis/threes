use super::agent_runner::Agent;
use super::board;
use super::game;
use super::random_agent::RandomAgent;
use super::utils;

// use histogram;
use rand::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;

type StandardFormBoard = board::Board;

type ActionRewards = crate::EnumMap<board::Direction, f64>;
//const EMPTY_ACTION_REWARDS : ActionRewards =
const LEARNING_RATE: f64 = 0.6;
const DISCOUNT_FACTOR: f64 = 0.1;
const EXPLORATION_RATE: f64 = 0.1;

pub struct QTable {
    action_rewards: HashMap<StandardFormBoard, ActionRewards>,
}

pub struct QAgent {
    rng: StdRng,
    random_agent: RandomAgent,
    q_table: QTable,
    learning_rate: f64,
    discount_factor: f64,
    exploration_rate: f64,
}

impl QTable {
    pub fn new() -> QTable {
        QTable {
            action_rewards: HashMap::new(),
        }
    }

    pub fn fold_cmp_directions(
        l_move: (board::Direction, f64),
        r_move: (board::Direction, f64),
    ) -> (board::Direction, f64) {
        match l_move.1.partial_cmp(&r_move.1) {
            None => panic!("NaN reward in action table"),
            Some(Ordering::Greater) => (l_move.0, l_move.1),
            _ => (r_move.0, r_move.1),
        }
    }

    pub fn max_action(&mut self, board: &board::Board) -> (board::Direction, f64) {
        self.get_action_rewards(board)
            .iter()
            .map(|(d, q)| (d, *q))
            .fold((board::Direction::Down, -1.0), Self::fold_cmp_directions)
    }

    pub fn q_value(&mut self, board: &board::Board, direction: board::Direction) -> f64 {
        self.get_action_rewards(board)[direction]
    }

    pub fn max_q_from_directions(
        &mut self,
        board: &board::Board,
        available_directions: &Vec<board::Direction>,
    ) -> board::Direction {
        let action_rewards = self.get_action_rewards(board);
        available_directions
            .iter()
            .map(|d| (*d, action_rewards[*d]))
            .fold((available_directions[0], -1.0), Self::fold_cmp_directions)
            .0
    }

    pub fn get_action_rewards(&mut self, board: &board::Board) -> &mut ActionRewards {
        self.action_rewards
            .entry(*board)
            .or_insert_with(|| crate::EnumMap::new())
    }
}

impl QAgent {
    pub fn new(seed: Option<&mut StdRng>) -> QAgent {
        let mut rng = utils::resolve_rng_from_seed(seed);
        let random_agent = RandomAgent::new(Some(&mut rng));
        QAgent {
            rng,
            random_agent,
            q_table: QTable::new(),
            learning_rate: LEARNING_RATE,
            discount_factor: DISCOUNT_FACTOR,
            exploration_rate: EXPLORATION_RATE,
        }
    }
}

impl Agent for QAgent {
    fn take_action(&mut self, game: &game::Game) -> board::Direction {
        if self.rng.gen_bool(self.exploration_rate) {
            // let's explore
            self.random_agent.take_action(game)
        } else {
            // take the best option
            self.q_table
                .max_q_from_directions(&game.cur_board, &game.available_moves())
        }
    }

    fn update(
        &mut self,
        board: &board::Board,
        action: board::Direction,
        new_board: &board::Board,
        reward: f64,
    ) {
        let new_q = self.q_table.q_value(board, action) * (1.0 - self.learning_rate)
            + self.learning_rate
                * (reward + self.discount_factor * self.q_table.max_action(new_board).1);
        self.q_table.get_action_rewards(board)[action] = new_q;
    }

    fn print(&self) {
        println!("qtable {} entries", self.q_table.action_rewards.len());
        let nonzero_q_value_counts = self
            .q_table
            .action_rewards
            .iter()
            // Count the number of non-zero q table entries for this board
            .map(|(b, acts)| {
                (
                    b,
                    acts,
                    acts.iter().filter(|(_d, q)| q.abs() > 0.001).count(),
                )
            })
            .collect::<Vec<(&board::Board, &ActionRewards, usize)>>();
        let q_counts = (0..5)
            .map(|fullness| {
                (
                    fullness,
                    nonzero_q_value_counts
                        .iter()
                        .filter(|(_b, _acts, c)| *c == fullness as usize)
                        .count(),
                )
            })
            .collect::<HashMap<i32, usize>>();
        println!("Q table fullness:\n{:?}", q_counts);
        let full_tables = nonzero_q_value_counts
            .iter()
            .filter(|(_b, _acts, c)| *c == 4)
            .collect::<Vec<&(&board::Board, &ActionRewards, usize)>>();
        //for (board, actions) in self.q_table.action_rewards.iter().take(3) {
        for (board, actions, _) in full_tables.iter().take(3) {
            println!("{}", board.simple_render());
            for d in &board::ALL_DIRECTIONS {
                println!("entry[{:?}] {}", d, actions[*d]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::agent_runner;
    use super::*;

    #[test]
    fn test_basic_qtable() {
        let mut table = QTable::new();
        let empty_board = board::Board::new();
        let mut test_actions = crate::EnumMap::new();
        test_actions[board::Direction::Down] = 42.0;
        table.action_rewards.insert(empty_board, test_actions);
        assert_eq!(
            table.max_action(&empty_board),
            (board::Direction::Down, 42.0)
        );
        // table.action_rewards[&empty_board] = test_actions;
    }

    #[test]
    fn test_basic_qagent() {
        let mut agent = QAgent::new(None);
        let board0 = board::Board::new();
        let mut board1 = board0;
        board1.set_value(0, 0, 1);
        let reward = 100.0;
        let action = board::Direction::Up;
        agent.update(&board0, action, &board1, reward);
        // Our updated reward in the q table is about 60
        assert!(agent.q_table.get_action_rewards(&board0)[action] - 60.0 < 0.01);
    }

    #[test]
    fn test_agent_play() {
        let mut agent = QAgent::new(None);
        let result = agent_runner::play_game(None, &mut agent);
        // We should play out a full game and always score more than 0
        assert_ne!(result.score, 0);
    }
}
