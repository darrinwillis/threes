use super::agent_runner::Agent;
use super::board;
use super::board::Direction;
use super::game;
use super::random_agent::RandomAgent;
use super::utils;

use rand::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;

type StandardFormBoard = board::Board;

type ActionRewards = crate::EnumMap<Direction, f64>;

pub struct RewardTable {
    rewards: ActionRewards,
    read_count: i64,
}

pub struct QTable {
    action_rewards: HashMap<StandardFormBoard, RewardTable>,
}

pub struct QAgent {
    rng: StdRng,
    random_agent: RandomAgent,
    q_table: QTable,
    learning_rate: f64,
    discount_factor: f64,
    exploration_rate: f64,
}

impl RewardTable {
    pub fn _new() -> RewardTable {
        RewardTable {
            rewards: crate::EnumMap::default(),
            read_count: 0,
        }
    }

    pub fn from_map(map: ActionRewards) -> RewardTable {
        RewardTable {
            rewards: map,
            read_count: 0,
        }
    }
}

impl QTable {
    pub fn new() -> QTable {
        QTable {
            action_rewards: HashMap::new(),
        }
    }

    pub fn fold_cmp_directions(
        l_move: (Direction, f64),
        r_move: (Direction, f64),
    ) -> (Direction, f64) {
        match l_move.1.partial_cmp(&r_move.1) {
            None => panic!("NaN reward in action table"),
            Some(Ordering::Greater) => (l_move.0, l_move.1),
            _ => (r_move.0, r_move.1),
        }
    }

    pub fn max_action(&mut self, board: &board::Board) -> (Direction, f64) {
        self.get_reward_table(board)
            .rewards
            .iter()
            .map(|(d, q)| (d, *q))
            .fold((Direction::Down, -1.0), Self::fold_cmp_directions)
    }

    pub fn q_value(&mut self, board: &board::Board, direction: Direction) -> f64 {
        self.get_reward_table(board).rewards[direction]
    }

    pub fn max_q_from_directions(
        &mut self,
        board: &board::Board,
        available_directions: &[Direction],
    ) -> Direction {
        let action_rewards = self.get_reward_table(board);
        available_directions
            .iter()
            .map(|d| (*d, action_rewards.rewards[*d]))
            .fold((available_directions[0], -1.0), Self::fold_cmp_directions)
            .0
    }

    pub fn get_reward_table(&mut self, board: &board::Board) -> &mut RewardTable {
        let mut reward_table = self.action_rewards.entry(*board).or_insert_with(|| {
            RewardTable::from_map({
                enum_map! {
                    Direction::Left => 80.0,
                    Direction::Up => 40.0,
                    Direction::Down => -40.0,
                    Direction::Right => -180.0,
                }
            })
        });
        reward_table.read_count += 1;
        reward_table
    }
}

impl QAgent {
    pub fn new(
        seed: Option<&mut StdRng>,
        learning_rate: f64,
        discount_factor: f64,
        exploration_rate: f64,
    ) -> QAgent {
        let mut rng = utils::resolve_rng_from_seed(seed);
        let random_agent = RandomAgent::new(Some(&mut rng));
        QAgent {
            rng,
            random_agent,
            q_table: QTable::new(),
            learning_rate,
            discount_factor,
            exploration_rate,
        }
    }
}

impl Agent for QAgent {
    fn take_action(&mut self, game: &game::Game, train_mode: bool) -> Direction {
        if train_mode && self.rng.gen_bool(self.exploration_rate) {
            // let's explore
            self.random_agent.take_action(game, train_mode)
        } else {
            // take the best option
            self.q_table
                .max_q_from_directions(&game.cur_board, &game.available_moves())
        }
    }

    fn update(
        &mut self,
        board: &board::Board,
        action: Direction,
        new_board: &board::Board,
        reward: f64,
    ) {
        let new_q = self.q_table.q_value(board, action) * (1.0 - self.learning_rate)
            + self.learning_rate
                * (reward + self.discount_factor * self.q_table.max_action(new_board).1);
        self.q_table.get_reward_table(board).rewards[action] = new_q;
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
                    acts.rewards
                        .iter()
                        .filter(|(_d, q)| q.abs() > 0.001)
                        .count(),
                )
            })
            .collect::<Vec<(&board::Board, &RewardTable, usize)>>();
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
            .collect::<Vec<&(&board::Board, &RewardTable, usize)>>();

        //for (board, actions) in self.q_table.action_rewards.iter().take(3) {
        for (board, actions, _) in full_tables.iter().take(3) {
            println!("{}", board.simple_render());
            for d in &board::ALL_DIRECTIONS {
                println!("entry[{:?}] {}", d, actions.rewards[*d]);
            }
        }

        println!("===========================\n\n\n");

        // All of the above might not be useful since "fullness" real means we have some priors.
        // I've changed my thinking here and basic priors seems like a good idea for now.

        // Instead, let's look at 'hot' tables. We track read count for each reward table. Let's
        // just look at the top few
        let num_top_tables = 10;

        let mut reward_tables: Vec<(&board::Board, &RewardTable)> =
            self.q_table.action_rewards.iter().collect();

        reward_tables
            .sort_unstable_by(|(_, acts_x), (_, acts_y)| acts_x.read_count.cmp(&acts_y.read_count));

        let mut histogram = histogram::Histogram::new();
        for table in reward_tables.iter() {
            histogram.increment(table.1.read_count as u64).unwrap();
        }
        println!("distribution of read counts:");
        println!("percentile, read count");
        for p in &[1.0, 10.0, 50.0, 90.0, 99.0, 99.9, 99.99] {
            println!(" p{:5}: {}", p, histogram.percentile(*p).unwrap());
        }
        println!("------------------------\n");

        for (ii, table) in reward_tables.iter().rev().take(num_top_tables).enumerate() {
            println!("#{} table [{} reads]", ii, table.1.read_count);
            println!("{}", table.0.simple_render());
            for d in &board::ALL_DIRECTIONS {
                println!("entry[{:?}] {}", d, table.1.rewards[*d]);
            }
            println!("------------------------\n");
        }

        let top_table = self
            .q_table
            .action_rewards
            .iter()
            .max_by(|(_, acts_x), (_, acts_y)| acts_x.read_count.cmp(&acts_y.read_count))
            .unwrap();
        println!("Top table [{} reads]", top_table.1.read_count);
        println!("{}", top_table.0.simple_render());
        for d in &board::ALL_DIRECTIONS {
            println!("entry[{:?}] {}", d, top_table.1.rewards[*d]);
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
        let mut test_actions = crate::EnumMap::default();
        test_actions[Direction::Down] = 42.0;
        let reward_table = RewardTable::from_map(test_actions);
        table.action_rewards.insert(empty_board, reward_table);
        assert_eq!(table.max_action(&empty_board), (Direction::Down, 42.0));
        // table.action_rewards[&empty_board] = test_actions;
    }

    #[test]
    fn test_basic_qagent() {
        let mut agent = QAgent::new(None, 0.5, 0.9, 0.1);
        let board0 = board::Board::new();
        let mut board1 = board0;
        board1.set_value(0, 0, 1);
        let reward = 100.0;
        let action = Direction::Up;
        let old_reward = agent.q_table.get_reward_table(&board0).rewards[action];
        agent.update(&board0, action, &board1, reward);
        // Our updated reward in the q table is about 60
        let new_reward = agent.q_table.get_reward_table(&board0).rewards[action];
        assert_ne!(old_reward, new_reward);
    }

    #[test]
    fn test_agent_play() {
        let mut agent = QAgent::new(None, 0.5, 0.9, 0.1);
        let result = agent_runner::play_game(None, &mut agent, true);
        // We should play out a full game and always score more than 0
        assert_ne!(result.score, 0);
    }
}
