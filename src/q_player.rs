use super::board;
use std::cmp::Ordering;
use std::collections::HashMap;

type ActionRewards = crate::EnumMap<board::Direction, f32>;
//const EMPTY_ACTION_REWARDS : ActionRewards =
const LEARNING_RATE: f32 = 0.6;
const DISCOUNT_FACTOR: f32 = 0.1;

pub struct QTable {
    action_rewards: HashMap<board::Board, ActionRewards>,
}

pub struct QAgent {
    q_table: QTable,
    learning_rate: f32,
    discount_factor: f32,
}

impl QTable {
    pub fn new() -> QTable {
        QTable {
            action_rewards: HashMap::new(),
        }
    }

    pub fn max_action(&mut self, board: &board::Board) -> (board::Direction, f32) {
        self.get_action_rewards(board).iter().fold(
            (board::Direction::Down, -1.0),
            |(old_direction, old_reward), (direction, &reward)| match old_reward
                .partial_cmp(&reward)
            {
                None => panic!("NaN reward in action table"),
                Some(Ordering::Less) => (direction, reward),
                _ => (old_direction, old_reward),
            },
        )
    }

    pub fn get_action_rewards(&mut self, board: &board::Board) -> &mut ActionRewards {
        self.action_rewards
            .entry(*board)
            .or_insert_with(|| crate::EnumMap::new())
    }
}

impl QAgent {
    pub fn new() -> QAgent {
        QAgent {
            q_table: QTable::new(),
            learning_rate: LEARNING_RATE,
            discount_factor: DISCOUNT_FACTOR,
        }
    }

    pub fn update(
        &mut self,
        board: &board::Board,
        action: board::Direction,
        new_board: &board::Board,
        reward: f32,
    ) {
        print!("updating");
        let new_q = self.q_table.get_action_rewards(board)[action] * (1.0 - self.learning_rate)
            + self.learning_rate
                * (reward + self.discount_factor * self.q_table.max_action(new_board).1);
        self.q_table.get_action_rewards(board)[action] = new_q;
    }
}

#[cfg(test)]
mod tests {
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
        let mut agent = QAgent::new();
        let board0 = board::Board::new();
        let mut board1 = board0;
        board1.set_value(0, 0, 1);
        let reward = 100.0;
        let action = board::Direction::Up;
        agent.update(&board0, action, &board1, reward);
        // Our updated reward in the q table is about 60
        assert!(agent.q_table.get_action_rewards(&board0)[action] - 60.0 < 0.01);
    }
}
