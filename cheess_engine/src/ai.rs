use crate::{GameState};

struct BigBrain<'a> {
  game_state: &'a GameState,
}

impl<'a> BigBrain<'a> {
  fn new(game_state: &'a GameState) -> Self {
    BigBrain {
      game_state
    }
  }
}

// if move == better {do that}