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
use crate::respond;
use crate::routes::DB;

#[derive(Deserialize)]
pub struct LoginBody {
  username: String,
  password: String,
}

#[derive(Serialize)]
#[cfg_attr(test, derive(Deserialize))]
pub struct LoginResponse {
  pub token: String,
  pub licensed: bool,
  pub incoming_message: Option<Vec<String>>,
}

pub async fn login(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  use crate::schema::sessions::dsl::*;
  use crate::schema::users::dsl::*;

  // Parse Login Body
  let body = hyper::body::to_bytes(req.into_body()).await.unwrap();
  let login_body = serde_json::from_slice(&body);
  if let Err(err) = login_body {
    return respond!(StatusCode::BAD_REQUEST, err.to_string());
  }
  let login_body: LoginBody = login_body.unwrap();

  // Find user if exists
  let db = DB.lock().await;
  let result = users.filter(username.eq(&login_body.username)).first(&*db);
  if let Err(_) = result {
    return respond!(StatusCode::BAD_REQUEST, "Invalid login credentials");
  }

  // Check password
  let user: User = result.unwrap();
  let parsed_hash = PasswordHash::new(&user.password_digest).unwrap();
  if Argon2::default()
    .verify_password(login_body.password.as_bytes(), &parsed_hash)
    .is_ok()
  {
    // Create session if password was correct
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
        // Return login message
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
        respond!(StatusCode::INTERNAL_SERVER_ERROR, "")
      },
    }
  } else {
    return respond!(StatusCode::BAD_REQUEST, "Invalid login credentials");
  }
}

#[cfg(test)]
mod test {
    use hyper::{Method, StatusCode};

    use crate::routes::{users::{test::before_user_test, register::UserBody}, test::build_test_request, handle_requests};

    use super::LoginResponse;


  #[tokio::test]
  async fn valid_user_auth_flow() {
    before_user_test().await;

    let value: UserBody = UserBody {
      name: "Tester McTester".to_string(),
      username: "tester".to_string(),
      password: "testtesttest".to_string(),
    };
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
  async fn invalid_login() {
    before_user_test().await;

    let value: UserBody = UserBody {
      name: "Tester McTester".to_string(),
      username: "tester".to_string(),
      password: "testtesttest".to_string(),
    };

    // User is never created

    let req = build_test_request(
      Method::POST,
      "/login",
      serde_json::to_string(&value).unwrap().as_str(),
      None,
    );
    let res = handle_requests(req).await.unwrap();
    let status = res.status();
    let body = String::from_utf8(hyper::body::to_bytes(res.into_body()).await.unwrap().to_vec()).unwrap();
    assert_eq!(status, StatusCode::BAD_REQUEST, "Test failed: {}", body);
  }

  #[tokio::test]
  async fn bad_password() {
    before_user_test().await;

    let value: UserBody = UserBody {
      name: "Tester McTester".to_string(),
      username: "tester".to_string(),
      password: "testtesttest".to_string(),
    };
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

    let value: UserBody = UserBody {
      name: "Tester McTester".to_string(),
      username: "tester".to_string(),
      password: "wrongpassword".to_string(),
    };

    let req = build_test_request(
      Method::POST,
      "/login",
      serde_json::to_string(&value).unwrap().as_str(),
      None,
    );
    let res = handle_requests(req).await.unwrap();
    let status = res.status();
    let body = String::from_utf8(hyper::body::to_bytes(res.into_body()).await.unwrap().to_vec()).unwrap();
    assert_eq!(status, StatusCode::BAD_REQUEST, "Test failed: {}", body);
  }

  #[tokio::test]
  async fn malformed_json() {

    let req = build_test_request(
      Method::POST,
      "/login",
      "bad Body",
      None,
    );
    let res = handle_requests(req).await.unwrap();
    let status = res.status();
    let body = String::from_utf8(hyper::body::to_bytes(res.into_body()).await.unwrap().to_vec()).unwrap();
    assert_eq!(status, StatusCode::BAD_REQUEST, "Test failed: {}", body);
  }
}
