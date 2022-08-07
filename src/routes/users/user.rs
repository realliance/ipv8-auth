use std::convert::Infallible;

use hyper::{Body, Request, Response, StatusCode};
use serde::Serialize;

use crate::routes::util::get_user_by_auth_header;
use crate::routes::DB;

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
    Ok((user, _)) => {
      Ok(
        Response::builder()
          .status(StatusCode::OK)
          .header("Content-Type", "application/json")
          .body(Body::from(
            serde_json::to_string(&UserResult {
              id: user.id.to_string(),
              licensed: user.is_licensed(),
              name: user.name,
              username: user.username,
            })
            .unwrap(),
          ))
          .unwrap(),
      )
    },
    Err(err) => Ok(err),
  }
}

#[cfg(test)]
mod test {
    use hyper::{Method, StatusCode};

    use crate::routes::{handle_requests, test::build_test_request};

  #[tokio::test]
  async fn fake_auth_header() {
    let req = build_test_request(Method::GET, "/user", "", Some("1234".to_string()));
    let res = handle_requests(req).await.unwrap();
    let status = res.status();
    let body = String::from_utf8(hyper::body::to_bytes(res.into_body()).await.unwrap().to_vec()).unwrap();
    assert_eq!(status, StatusCode::UNAUTHORIZED, "Test failed: {}", body);
  }
}
