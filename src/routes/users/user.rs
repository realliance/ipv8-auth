use std::convert::Infallible;

use hyper::{Body, Request, Response, StatusCode};
use serde::Serialize;

use crate::routes::{util::get_user_by_auth_header, DB};

#[derive(Serialize)]
pub struct UserResult {
  id: String,
  name: String,
  username: String,
  licensed: bool,
}

pub async fn get_user_by_token(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let db = DB.lock().await;

  match get_user_by_auth_header(&db, &req) {
    Ok((user, _)) => Ok(
      Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
          Body::from(
            serde_json::to_string(&UserResult {
              id: user.id.to_string(),
              licensed: user.is_licensed(),
              name: user.name,
              username: user.username,
            })
            .unwrap(),
          ),
        )
        .unwrap(),
    ),
    Err(err) => Ok(err),
  }
}
