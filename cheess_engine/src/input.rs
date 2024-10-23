use crate::{Event, Handler, Payload, GameMode};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct UserInput {
  pub input_queue: VecDeque<InputType>,
}
#[derive(Debug)]
pub enum InputType {
  Exit,
  Resign,
  Reset,
  GameMode(GameMode),
  Move(String),
}

impl Handler for UserInput {
  fn handle_mut(&mut self, _event: Event, _payload: Payload) {

  }
}


// pub fn special_commands(input: &str) {}