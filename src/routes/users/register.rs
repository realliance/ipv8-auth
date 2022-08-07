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
  pub const SHORT_USERNAME_ERR: &'static str = "Username must be at least 3 characters long";
  pub const SHORT_PASSWORD_ERR: &'static str = "Password must be at least 12 characters long";
  pub const LONG_USERNAME_ERR: &'static str = "Username must be at most 100 characters long";
  pub const SHORT_COMPANY_ERR: &'static str = "Company Name must be at least 3 characters long";
  pub const LONG_COMPANY_ERR: &'static str = "Company Name must be at most 100 characters long";
  pub const USERNAME_NONALPHABETIC_ERR: &'static str = "Username must be alphabetic";

  pub fn is_valid(&self) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if self.username.len() < 3 {
      errors.push(Self::SHORT_USERNAME_ERR.to_string());
    }

    if self.username.len() > 100 {
      errors.push(Self::LONG_USERNAME_ERR.to_string());
    }

    if self.name.len() < 3 {
      errors.push(Self::SHORT_COMPANY_ERR.to_string());
    }

    if self.name.len() > 100 {
      errors.push(Self::LONG_COMPANY_ERR.to_string());
    }

    let alphabetic_check = self.username.chars().all(|c| c.is_ascii_alphanumeric());
    if !alphabetic_check {
      errors.push(Self::USERNAME_NONALPHABETIC_ERR.to_string());
    }

    if self.password.len() < 12 {
      errors.push(Self::SHORT_PASSWORD_ERR.to_string());
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
  use crate::routes::users::test::{before_user_test};

  async fn assert_bad_registration(value: UserBody, contained_errors: Vec<String>) {
    before_user_test().await;

    let req = build_test_request(
      Method::POST,
      "/register",
      serde_json::to_string(&value).unwrap().as_str(),
      None,
    );
    let res = handle_requests(req).await.unwrap();
    let status = res.status();
    let body = String::from_utf8(hyper::body::to_bytes(res.into_body()).await.unwrap().to_vec()).unwrap();
    assert_eq!(
      status,
      StatusCode::BAD_REQUEST,
      "Test failed: {}",
      body
    );

    contained_errors.iter().for_each(|err| {
      assert!(body.contains(err), "Failed, error list did not contain \"{}\", only: {}", err, body);
    });
  }

  #[tokio::test]
  async fn invalid_user_bad_password() {
    let value: UserBody = UserBody {
      name: "Tester McTester".to_string(),
      username: "tester".to_string(),
      password: "testtest".to_string(),
    };

    assert_bad_registration(value, vec![UserBody::SHORT_PASSWORD_ERR.to_string()]).await;
  }

  #[tokio::test]
  async fn invalid_user_short_username() {
    let value: UserBody = UserBody {
      name: "Tester McTester".to_string(),
      username: "aa".to_string(),
      password: "testtesttest".to_string(),
    };

    assert_bad_registration(value, vec![UserBody::SHORT_USERNAME_ERR.to_string()]).await;
  }

  #[tokio::test]
  async fn invalid_user_long_username() {
    let value: UserBody = UserBody {
      name: "Tester McTester".to_string(),
      username: String::from_utf8(vec![b'a'; 1000]).unwrap(),
      password: "testtesttest".to_string(),
    };

    assert_bad_registration(value, vec![UserBody::LONG_USERNAME_ERR.to_string()]).await;
  }

  #[tokio::test]
  async fn invalid_user_long_company() {
    let value: UserBody = UserBody {
      name: String::from_utf8(vec![b'a'; 1000]).unwrap(),
      username: "testtest".to_string(),
      password: "testtesttest".to_string(),
    };

    assert_bad_registration(value, vec![UserBody::LONG_COMPANY_ERR.to_string()]).await;
  }

  #[tokio::test]
  async fn invalid_user_short_company() {
    let value: UserBody = UserBody {
      name: "a".to_string(),
      username: "testtest".to_string(),
      password: "testtesttest".to_string(),
    };

    assert_bad_registration(value, vec![UserBody::SHORT_COMPANY_ERR.to_string()]).await;
  }

  #[tokio::test]
  async fn invalid_user_nonalphabetic_company() {
    let value: UserBody = UserBody {
      name: "Tester McTester".to_string(),
      username: "$$$$".to_string(),
      password: "testtesttest".to_string(),
    };

    assert_bad_registration(value, vec![UserBody::USERNAME_NONALPHABETIC_ERR.to_string()]).await;
  }

  #[tokio::test]
  async fn invalid_registration_malformed() {
    let req = build_test_request(
      Method::POST,
      "/register",
      "Bad body",
      None,
    );
    let res = handle_requests(req).await.unwrap();
    let status = res.status();
    let body = String::from_utf8(hyper::body::to_bytes(res.into_body()).await.unwrap().to_vec()).unwrap();
    assert_eq!(status, StatusCode::BAD_REQUEST, "Test failed: {}", body);
  }
}
