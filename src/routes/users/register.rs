use std::convert::Infallible;

use hyper::{Body, Request, Response, StatusCode};
use serde::Deserialize;
#[cfg(test)]
use serde::Serialize;
use tracing::error;

use crate::models::create_user;
use crate::respond;
use crate::routes::DB;

#[derive(Deserialize, Clone)]
#[cfg_attr(test, derive(Serialize))]
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

    // TODO Check if username is taken

    if errors.len() > 0 {
      Err(errors)
    } else {
      Ok(())
    }
  }
}

pub async fn register_user(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  // Parse Body
  let body = hyper::body::to_bytes(req.into_body()).await.unwrap();
  let user_body = serde_json::from_slice(&body);
  if let Err(err) = user_body {
    return respond!(StatusCode::BAD_REQUEST, err.to_string());
  }
  let user_body: UserBody = user_body.unwrap();

  // Determine if user is valid
  let db = DB.lock().await;
  if let Err(errors) = user_body.is_valid() {
    return respond!(StatusCode::BAD_REQUEST, serde_json::to_string(&errors).unwrap());
  }

  // TODO Check if user already exists.

  // Create new user
  let user = create_user(&db, user_body.name, user_body.username, user_body.password, 0);
  if let Err(err) = user {
    error!("{}", err.to_string());
    return respond!(StatusCode::INTERNAL_SERVER_ERROR, "");
  }

  respond!(StatusCode::OK, "")
}

#[cfg(test)]
mod test {
  use hyper::{Method, StatusCode};

  use super::UserBody;
  use crate::routes::handle_requests;
  use crate::routes::test::build_test_request;
  use crate::routes::users::login::LoginResponse;
  use crate::routes::users::test::{before_user_test, INVALID_USER_BAD_PASSWORD, VALID_USER};

  #[tokio::test]
  async fn valid_user_auth_flow() {
    before_user_test().await;

    let value: UserBody = VALID_USER.clone();
    let req = build_test_request(
      Method::POST,
      "/register",
      serde_json::to_string(&value).unwrap().as_str(),
      None,
    );
    let res = handle_requests(req).await.unwrap();
    assert_eq!(
      res.status(),
      StatusCode::OK,
      "Request failed: {}",
      String::from_utf8(hyper::body::to_bytes(res.into_body()).await.unwrap().to_vec()).unwrap()
    );

    let req = build_test_request(
      Method::POST,
      "/login",
      serde_json::to_string(&value).unwrap().as_str(),
      None,
    );
    let res = handle_requests(req).await.unwrap();
    let status = res.status();
    let body = String::from_utf8(hyper::body::to_bytes(res.into_body()).await.unwrap().to_vec()).unwrap();
    assert_eq!(status, StatusCode::OK, "Request failed: {}", body);

    let response_obj: LoginResponse = serde_json::from_str(&body).unwrap();
    let req = build_test_request(Method::GET, "/user", "", Some(response_obj.token));
    let res = handle_requests(req).await.unwrap();
    let status = res.status();
    let body = String::from_utf8(hyper::body::to_bytes(res.into_body()).await.unwrap().to_vec()).unwrap();
    assert_eq!(status, StatusCode::OK, "Request failed: {}", body);
  }

  #[tokio::test]
  async fn invalid_user_registration() {
    before_user_test().await;

    let value: UserBody = INVALID_USER_BAD_PASSWORD.clone();
    let req = build_test_request(
      Method::POST,
      "/register",
      serde_json::to_string(&value).unwrap().as_str(),
      None,
    );
    let res = handle_requests(req).await.unwrap();
    let status = res.status();
    let body = res.into_body();
    assert_eq!(
      status,
      StatusCode::BAD_REQUEST,
      "Test failed: {}",
      String::from_utf8(hyper::body::to_bytes(body).await.unwrap().to_vec()).unwrap()
    );
  }
}
