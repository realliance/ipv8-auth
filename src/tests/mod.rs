use hyper::{Body, Request, StatusCode, Method};

use crate::routes::handle_requests;

mod users;

fn build_test_request<'a>(verb: Method, path: &'a str, body: &'a str) -> Request<Body> {
  Request::builder()
    .method(verb)
    .uri(path)
    .body(Body::from(body.to_string()))
    .unwrap()
}

#[tokio::test]
async fn can_receive_404() {
  let res = handle_requests(build_test_request(Method::GET, "/fakepath", "")).await.unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn can_probe_health() {
  let res = handle_requests(build_test_request(Method::GET, "/health", "")).await.unwrap();
  assert_eq!(res.status(), StatusCode::OK);
}
