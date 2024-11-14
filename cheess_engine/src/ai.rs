use crate::{get_valid_moves_for_piece, simulate_move, take_turn, BoardRep, GameState, Move, PlayerValidMoves};
use rand::Rng;
use std::sync::{Arc, Mutex};

pub struct BigBrain {
  game_state: Arc<Mutex<GameState>>,
  board: BoardRep,
}

impl BigBrain {
  pub fn new(game_state: Arc<Mutex<GameState>>) -> Self {
    let lock = game_state.lock().unwrap().board.clone();
    BigBrain {
        game_state: game_state,
        board: lock,
      }
      
  }
  
  
  fn choose_move(&self) -> Move {
    let lock = self.game_state.lock().unwrap();
    let rand_index = rand::thread_rng().gen_range(0..lock.move_list.black.len());
    lock.move_list.black[rand_index]
  }

  pub fn ai_make_move(&mut self) {
    if self.game_state.lock().unwrap().player_turn == 2 {
      let translation = self.choose_move();
      println!("Chosen Move: {:?}", translation);
      take_turn(&mut *self.game_state.lock().unwrap(), translation);
    }

  }
}

// if move == better {do that}
