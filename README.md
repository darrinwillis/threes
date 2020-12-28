# threes

This is a minimal implementation of the block-combining puzzle game Threes for use as a platform to test game playing algorithms.

## State as of 12/28/20

### Game
The game logic supports randomly spawning blocks 1 or 2 but no larger numbers. The game also attempts to keep parity between the total number of 1s and 2s so that the game does not become unplayable if, for example, the board gets filled with 1s with no 2s to combine with.

There is only simple board printing and statistics reporting.

### Agents
There is a simple random walk agent.

There is a Q agent implementation which keeps a score table for each board state and is able to learn the game, but very slowly. The slowness is likely due to how inefficiently the agent is able to backpropogate learnings from "deep" board states to earlier ones. It also does not use the game simulator to do any lookhead or graph search.

### Next steps
- Add data analytics to understand evolution of the agent
   - For a given board, graph action scores across a learning period
   - For a given learning session, graph the agent's final scores over the session
   - Graph the distribution of number of visitations of each board state
- Add lookahead to the agent so it explores the possible outcomes before taking an action
