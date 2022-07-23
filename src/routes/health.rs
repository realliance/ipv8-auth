use std::convert::Infallible;

use hyper::{StatusCode, Body, Response, Request, Method};
use futures::FutureExt;
use crate::{router::Routable, respond, route_func};


pub struct HealthRouter;

impl Routable for HealthRouter {
  fn routes(&self) -> Vec<crate::router::RoutedFunction> {
    vec![
      route_func!(Method::GET, "/health", post_next_instruction)
    ]
  }
}

pub async fn post_next_instruction(_: Request<Body>) -> Result<Response<Body>, Infallible> {
  respond!(StatusCode::OK, "OK")
}
