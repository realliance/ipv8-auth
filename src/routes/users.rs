use std::convert::Infallible;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use hyper::{Body, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::DB;
use crate::models::{create_user, Session, User};

#[derive(Deserialize)]
pub struct UserBody {
  pub name: String,
  pub email: String,
  pub password: String,
}

pub async fn register_user(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let body = hyper::body::to_bytes(req.into_body()).await.unwrap();
  let user_body = serde_json::from_slice(&body);
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
  let user = create_user(&db, &user_body.name, &user_body.email, &user_body.password);
  if let Err(err) = user {
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(err.to_string()))
        .unwrap(),
    );
  }

  Ok(Response::builder().status(200).body(Body::empty()).unwrap())
}

#[derive(Deserialize)]
pub struct LoginBody {
  email: String,
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
  let result = users.filter(email.eq(&login_body.email)).first(&*db);
  if let Err(err) = result {
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(err.to_string()))
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
      Err(err) => Ok(
        Response::builder()
          .status(StatusCode::BAD_REQUEST)
          .body(Body::from(err.to_string()))
          .unwrap(),
      ),
    }
  } else {
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from("Invalid password"))
        .unwrap(),
    );
  }
}
