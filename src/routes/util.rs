use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use hyper::{Body, Request, Response, StatusCode};
use tracing::warn;
use uuid::Uuid;

use crate::diesel::ExpressionMethods;
use crate::models::{update_session_last_used, Session, User};

#[macro_export]
macro_rules! respond {
  ($response_code:expr, $message:expr) => {
    Ok(
      Response::builder()
        .status($response_code)
        .body(Body::from($message))
        .unwrap(),
    )
  };
}

pub fn get_user_by_auth(db: &PgConnection, user_token: Uuid) -> Result<(User, Session), String> {
  use crate::schema::sessions::dsl::*;
  use crate::schema::users::dsl::*;

  let result = sessions.filter(token.eq(user_token)).first(&*db);
  if let Err(_) = result {
    return Err("Invalid Token".to_string());
  }

  // TODO expire session token if it's been too long.

  let session: Session = result.unwrap();
  if let Err(err) = update_session_last_used(&db, session.token) {
    warn!("Failed to update session last used token {}", err.to_string());
  }

  let result = users.filter(id.eq(session.user_id)).first(&*db);
  if result.is_err() {
    return Err("Invalid Token".to_string());
  }

  Ok((result.unwrap(), session))
}

pub fn get_user_by_auth_header(db: &PgConnection, req: &Request<Body>) -> Result<(User, Session), Response<Body>> {
  if req.headers().get("Authorization").is_none() {
    return Err(
      Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from("No authorization header"))
        .unwrap(),
    );
  }
  let user_token = Uuid::parse_str(req.headers().get("Authorization").unwrap().to_str().unwrap());
  if user_token.is_err() {
    return Err(
      Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from("Invalid authorization header"))
        .unwrap(),
    );
  }
  let user_token = user_token.unwrap();

  get_user_by_auth(db, user_token).map_err(|e|
    Response::builder()
      .status(StatusCode::BAD_REQUEST)
      .body(Body::from(e))
      .unwrap())
}
