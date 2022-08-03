use std::borrow::Cow;

use hyper::{Body, Request, StatusCode};

use crate::routes::handle_requests;


fn build_test_request<'a>(path: &'a str, body: &'a str) -> Request<Body> {
  Request::builder()
    .uri(path)
    .body(Body::from(body.to_string()))
    .unwrap()
}

#[tokio::test]
async fn can_receive_404() {
  let res = handle_requests(build_test_request("/fakepath", "")).await.unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn can_probe_health() {
  let res = handle_requests(build_test_request("/health", "")).await.unwrap();
  assert_eq!(res.status(), StatusCode::OK);
}
