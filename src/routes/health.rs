use std::convert::Infallible;

use futures::FutureExt;
use hyper::{Body, Method, Request, Response, StatusCode};

use crate::router::Routable;
use crate::{respond, route_func};

pub struct HealthRouter;

impl Routable for HealthRouter {
  fn routes(&self) -> Vec<crate::router::RoutedFunction> {
    vec![route_func!(Method::GET, "/health", post_next_instruction)]
  }
}

pub async fn post_next_instruction(_: Request<Body>) -> Result<Response<Body>, Infallible> {
  respond!(StatusCode::OK, "OK")
}

#[cfg(test)]
mod test {
  use hyper::{Method, StatusCode};

  use crate::routes::handle_requests;
  use crate::routes::test::build_test_request;

  #[tokio::test]
  async fn can_probe_health() {
    let res = handle_requests(build_test_request(Method::GET, "/health", "", None))
      .await
      .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
  }
}
