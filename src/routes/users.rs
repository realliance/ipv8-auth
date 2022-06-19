use std::convert::Infallible;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use hyper::{Body, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use super::DB;
use crate::models::{create_user, update_session_last_used, Session, User};

#[derive(Deserialize)]
pub struct UserBody {
  pub name: String,
  pub username: String,
  pub password: String,
}

impl UserBody {
  pub fn is_valid(&self) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if self.username.len() < 3 {
      errors.push("Username must be at least 3 characters long".to_string());
    }

    if self.username.len() > 100 {
      errors.push("Username must be at most 100 characters long".to_string());
    }

    if self.name.len() < 3 {
      errors.push("Company Name must be at least 3 characters long".to_string());
    }

    if self.name.len() > 100 {
      errors.push("Company Name must be at most 100 characters long".to_string());
    }

    let alphabetic_check = self.username.chars().all(|c| c.is_alphanumeric());
    if !alphabetic_check {
      errors.push("Username must be alphabetic".to_string());
    }

    if self.password.len() < 12 {
      errors.push("Password must be at least 12 characters long".to_string());
    }

    if errors.len() > 0 {
      Err(errors)
    } else {
      Ok(())
    }
  }
}

pub async fn register_user(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let body = hyper::body::to_bytes(req.into_body()).await.unwrap();
  let user_body = serde_json::from_slice(&body);

  // Bad Request if the body is not valid JSON
  if let Err(err) = user_body {
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(err.to_string()))
        .unwrap(),
    );
  }
  let user_body: UserBody = user_body.unwrap();

  let db = DB.lock().await;
  if let Err(errors) = user_body.is_valid() {
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(serde_json::to_string(&errors).unwrap()))
        .unwrap(),
    );
  }

  let user = create_user(&db, user_body.name, user_body.username, user_body.password);
  if let Err(err) = user {
    error!("{}", err.to_string());
    return Ok(
      Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::empty())
        .unwrap(),
    );
  }

  Ok(Response::builder().status(200).body(Body::empty()).unwrap())
}

#[derive(Deserialize)]
pub struct LoginBody {
  username: String,
  password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
  token: String,
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
        Ok(
          Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(
              serde_json::to_string(&LoginResponse {
                token: session.token.to_string(),
              })
              .unwrap(),
            ))
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
  update_session_last_used(&db, session.token);

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
