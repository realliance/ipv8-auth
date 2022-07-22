use serde::{Deserialize, Serialize};

use crate::util::get_server_url;

#[derive(Serialize, Deserialize)]
pub struct GameStrings {
  pub welcome_begin: String,
  pub puzzle_prompt: String,
  pub welcome_end: String,
}

impl GameStrings {
  pub fn puzzle_message(&self) -> String {
    self.welcome_begin.clone() + &self.puzzle_prompt + &self.welcome_end
  }
}

lazy_static::lazy_static! {
  pub static ref GAME_STRINGS: GameStrings = toml::from_str(&include_str!("../strings.toml").replace("%URL%", &get_server_url())).unwrap();
}
