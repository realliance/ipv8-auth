use std::convert::Infallible;

use hyper::{Request, Body, Response, StatusCode};
use serde::Deserialize;
use uuid::Uuid;

use crate::{models::{update_fizz, update_buzz, update_instructions}, respond, routes::{util::get_user_by_auth_header, DB}};

#[derive(Deserialize)]
struct TokenBody {
  pub token: String,
}

async fn read_body_for_token(req: Request<Body>) -> Result<Uuid, Result<Response<Body>, Infallible>> {
  let body = hyper::body::to_bytes(req.into_body()).await.unwrap();
  let token_body = serde_json::from_slice(&body);
  if let Err(err) = token_body {
    return Err(respond!(StatusCode::BAD_REQUEST, err.to_string()));
  }
  let token_body: TokenBody = token_body.unwrap();
  match Uuid::parse_str(token_body.token.as_str()) {
    Ok(uuid) => Ok(uuid),
    Err(err) => Err(respond!(StatusCode::BAD_REQUEST, err.to_string())),
  }
}

pub async fn post_fizz(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let db = DB.lock().await;
  
  let user = get_user_by_auth_header(&db, &req);
  if let Err(res) = user {
    return Ok(res);
  }
  let (user, _) = user.unwrap();

  let token = read_body_for_token(req).await;
  if let Err(err) = token {
    return err;
  }
  let token = token.unwrap();

  match update_fizz(&db, user.id, token) {
    Ok(_) => respond!(StatusCode::OK, "Fizz: Instruction Received!"),
    Err(_) => respond!(StatusCode::INTERNAL_SERVER_ERROR, "An internal error occured"),
  }
}

pub async fn post_buzz(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let db = DB.lock().await;
  
  let user = get_user_by_auth_header(&db, &req);
  if let Err(res) = user {
    return Ok(res);
  }
  let (user, _) = user.unwrap();

  let token = read_body_for_token(req).await;
  if let Err(err) = token {
    return err;
  }
  let token = token.unwrap();

  match update_buzz(&db, user.id, token) {
    Ok(_) => respond!(StatusCode::OK, "Buzz: Instruction Received!"),
    Err(_) => respond!(StatusCode::INTERNAL_SERVER_ERROR, "An internal error occured"),
  }
}

pub async fn post_instructions(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let db = DB.lock().await;
  
  let user = get_user_by_auth_header(&db, &req);
  if let Err(res) = user {
    return Ok(res);
  }
  let (user, _) = user.unwrap();

  let token = read_body_for_token(req).await;
  if let Err(err) = token {
    return err;
  }
  let token = token.unwrap();

  match update_instructions(&db, user.id, token) {
    Ok(_) => respond!(StatusCode::OK, "Instruction Received for Rescheduling"),
    Err(_) => respond!(StatusCode::INTERNAL_SERVER_ERROR, "An internal error occured"),
  }
}
