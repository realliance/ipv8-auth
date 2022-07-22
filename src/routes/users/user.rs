use std::convert::Infallible;

use diesel::QueryDsl;
use hyper::{Body, Request, Response, StatusCode};
use serde::Serialize;
use tracing::warn;
use uuid::Uuid;

use crate::diesel::{ExpressionMethods, RunQueryDsl};
use crate::models::{update_session_last_used, Session, User};
use crate::routes::DB;

#[derive(Serialize)]
pub struct UserResult {
  token: String,
  name: String,
  username: String,
}

pub async fn get_user_by_token(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  use crate::schema::sessions::dsl::*;
  use crate::schema::users::dsl::*;

  let db = DB.lock().await;
  if req.headers().get("Authorization").is_none() {
    return Ok(
      Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from("No authorization header"))
        .unwrap(),
    );
  }
  let user_token = Uuid::parse_str(req.headers().get("Authorization").unwrap().to_str().unwrap());
  if user_token.is_err() {
    return Ok(
      Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from("Invalid authorization header"))
        .unwrap(),
    );
  }
  let user_token = user_token.unwrap();

  let result = sessions.filter(token.eq(user_token)).first(&*db);
  if let Err(_) = result {
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from("Invalid token"))
        .unwrap(),
    );
  }

  let session: Session = result.unwrap();
  if let Err(err) = update_session_last_used(&db, session.token) {
    warn!("Failed to update session last used token {}", err.to_string());
  }

  let result = users.filter(id.eq(session.user_id)).first(&*db);
  if result.is_err() {
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from("Invalid token"))
        .unwrap(),
    );
  }

  let user: User = result.unwrap();

  return Ok(
    Response::builder()
      .status(StatusCode::OK)
      .header("Content-Type", "application/json")
      .body(
        Body::from(
          serde_json::to_string(&UserResult {
            token: session.token.to_string(),
            name: user.name,
            username: user.username,
          })
          .unwrap(),
        ),
      )
      .unwrap(),
  );
}
