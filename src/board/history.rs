use super::gamestate::GameState;
use crate::defs::MAX_GAME_MOVES;

#[derive(Clone, Debug)]
pub struct History {
    pub list: [GameState; MAX_GAME_MOVES],
    count: usize,
}

impl History {
    // Create a new history array containing game states.
    pub fn new() -> Self {
        Self {
            list: [GameState::new(); MAX_GAME_MOVES],
            count: 0,
        }
    }

    // Wipe the entire array.
    pub fn clear(&mut self) {
        self.list = [GameState::new(); MAX_GAME_MOVES];
        self.count = 0;
    }

    // Put a new game state into the array.
    pub fn push(&mut self, g: GameState) {
        self.list[self.count] = g;
        self.count += 1;
    }

    // Return the last game state and decremnt the counter. The game state is
    // not deleted from the array. If necessary, another game state will just
    // overwrite it.
    pub fn pop(&mut self) -> GameState {
        self.count -= 1;
        self.list[self.count]
    }

    pub fn get_ref(&self, index: usize) -> &GameState {
        &self.list[index]
    }

    pub fn len(&self) -> usize {
        self.count
    }
}
