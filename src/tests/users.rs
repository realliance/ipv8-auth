use crate::{diesel::RunQueryDsl, routes::users::{register::UserBody, user::UserResult, login::LoginResponse}};
use diesel::PgConnection;
use hyper::{StatusCode, Method};
use lazy_static::lazy_static;
use tokio::sync::MutexGuard;
use uuid::Uuid;

use crate::{
  models::User,
  routes::{handle_requests, DB},
  tests::build_test_request,
};

lazy_static! {
  static ref VALID_USER: UserBody = UserBody { 
    name: "Tester McTester".to_string(), 
    username: "tester".to_string(), 
    password: "testtesttest".to_string(),
  };
}

async fn before_user_test() {
  use crate::schema::users::dsl::users;
  use crate::schema::sessions::dsl::sessions;

  dotenv::dotenv().ok();
  let conn = DB.lock().await;
  diesel::delete(sessions).execute(&*conn).unwrap();
  diesel::delete(users).execute(&*conn).unwrap();
}

#[tokio::test]
async fn valid_user_auth_flow() {
  before_user_test().await;
  
  let value: UserBody = VALID_USER.clone();
  let req = build_test_request(Method::POST, "/register", serde_json::to_string(&value).unwrap().as_str(), None);
  let res = handle_requests(req).await.unwrap();
  assert_eq!(res.status(), StatusCode::OK, "Request failed: {}", String::from_utf8(hyper::body::to_bytes(res.into_body()).await.unwrap().to_vec()).unwrap());

  let req = build_test_request(Method::POST, "/login", serde_json::to_string(&value).unwrap().as_str(), None);
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
