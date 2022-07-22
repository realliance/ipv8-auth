use std::convert::Infallible;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use diesel::query_dsl::methods::FilterDsl;
use hyper::{Body, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::diesel::{ExpressionMethods, RunQueryDsl};
use crate::game::GAME_STRINGS;
use crate::models::{Session, User};
use crate::routes::DB;

#[derive(Deserialize)]
pub struct LoginBody {
  username: String,
  password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
  token: String,
  licensed: bool,
  incoming_message: Option<Vec<String>>,
}

pub async fn login(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  use crate::schema::sessions::dsl::*;
  use crate::schema::users::dsl::*;

  let body = hyper::body::to_bytes(req.into_body()).await.unwrap();
  let login_body = serde_json::from_slice(&body);
  if let Err(err) = login_body {
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(err.to_string()))
        .unwrap(),
    );
  }
  let login_body: LoginBody = login_body.unwrap();

  let db = DB.lock().await;
  let result = users.filter(username.eq(&login_body.username)).first(&*db);
  if let Err(_) = result {
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from("Invalid login credentials"))
        .unwrap(),
    );
  }

  let user: User = result.unwrap();
  let parsed_hash = PasswordHash::new(&user.password_digest).unwrap();
  if Argon2::default()
    .verify_password(login_body.password.as_bytes(), &parsed_hash)
    .is_ok()
  {
    let session = Session {
      token: Uuid::new_v4(),
      user_id: user.id,
      last_used: chrono::Utc::now().naive_utc(),
    };

    match diesel::insert_into(sessions)
      .values(&session)
      .get_result::<Session>(&*db)
    {
      Ok(session) => {
        let incoming_message = if !user.is_licensed() {
          Some(GAME_STRINGS.puzzle_message())
        } else {
          None
        };
        let incoming_message = incoming_message.map(|x| x.split('\n').map(|x| x.to_owned()).collect());
        Ok(
          Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(
              Body::from(
                serde_json::to_string(&LoginResponse {
                  token: session.token.to_string(),
                  licensed: user.is_licensed(),
                  incoming_message,
                })
                .unwrap(),
              ),
            )
            .unwrap(),
        )
      },
      Err(err) => {
        error!("{}", err.to_string());
        Ok(
          Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap(),
        )
      },
    }
  } else {
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from("Invalid login credentials"))
        .unwrap(),
    );
  }
}
