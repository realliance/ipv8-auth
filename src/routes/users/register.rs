use std::convert::Infallible;

use hyper::{Body, Request, Response, StatusCode};
use serde::Deserialize;
use tracing::error;

#[cfg(test)]
use serde::Serialize;

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
