use std::convert::Infallible;

use futures::FutureExt;
use hyper::{Body, Method, Request, Response, StatusCode};
use rand::Rng;
use serde::Serialize;

use self::resp::{post_buzz, post_fizz, post_instructions};
use super::util::get_user_by_auth_header;
use super::DB;
use crate::models::create_game_instruction;
use crate::router::{Routable, RoutedFunction};
use crate::{respond, route_func};

mod resp;

#[derive(Serialize)]
pub struct InstructionResponse {
  id: i32,
  token: String,
  correct_in_a_row: i32,
}

pub async fn post_next_instruction(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let db = DB.lock().await;

  let user = get_user_by_auth_header(&db, &req);
  if let Err(res) = user {
    return Ok(res);
  }
  let (user, _) = user.unwrap();
  if user.is_licensed() {
    return respond!(StatusCode::NO_CONTENT, "License Exam already passed!");
  }

  let mut rng = rand::thread_rng();
  let inst: u16 = rng.gen();
  match create_game_instruction(&db, user.id, inst) {
    Ok((user, game)) => {
      // Check if last instruction was enough
      if user.is_licensed() {
        return respond!(StatusCode::NO_CONTENT, "License Exam already passed!");
      }

      Ok(
        Response::builder()
          .status(StatusCode::OK)
          .header("Content-Type", "application/json")
          .body(
            Body::from(
              serde_json::to_string(&InstructionResponse {
                token: game.token.to_string(),
                id: game.instruction,
                correct_in_a_row: user.license_game_stage,
              })
              .unwrap(),
            ),
          )
          .unwrap(),
      )
    },
    Err(_) => respond!(StatusCode::INTERNAL_SERVER_ERROR, "Unknown error occured."),
  }
}

pub struct GameRouter;

impl Routable for GameRouter {
  fn routes(&self) -> Vec<RoutedFunction> {
    vec![
      route_func!(Method::POST, "/next_instruction", post_next_instruction),
      route_func!(Method::POST, "/instructions", post_instructions),
      route_func!(Method::POST, "/fizz", post_fizz),
      route_func!(Method::POST, "/buzz", post_buzz),
    ]
  }
}
