use std::convert::Infallible;
use std::env;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use diesel::{Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use dotenv::dotenv;
use hyper::{Body, Method, Request, Response, StatusCode};
use lazy_static::lazy_static;
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::info;

use crate::models::{create_user, User};

lazy_static! {
  static ref DB: Mutex<PgConnection> = Mutex::new(establish_connection());
}

fn establish_connection() -> PgConnection {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

#[derive(Deserialize)]
pub struct UserBody {
  pub name: String,
  pub email: String,
  pub password: String,
}

async fn register_user(req: Request<Body>) -> Result<Response<Body>, Infallible> {
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

async fn login(req: Request<Body>) -> Result<Response<Body>, Infallible> {
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
    return Ok(Response::builder().status(StatusCode::OK).body(Body::empty()).unwrap());
  } else {
    return Ok(
      Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from("Invalid password"))
        .unwrap(),
    );
  }
}

pub async fn handle_requests(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let (method, uri) = (req.method(), req.uri().path());
  info!("Request: {:?}", (method, uri));

  match (method, uri) {
    (&Method::POST, "/register") => register_user(req).await,
    (&Method::POST, "/login") => login(req).await,
    _ => Ok(Response::builder().status(404).body(Body::from("Not found")).unwrap()),
  }
}
