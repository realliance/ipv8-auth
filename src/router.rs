use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

use hyper::{Body, Method, Request, Response};
use tracing::info;

#[derive(PartialEq)]
pub struct Route(pub Method, pub Cow<'static, str>);

impl From<(Method, Cow<'static, str>)> for Route {
  fn from((method, path): (Method, Cow<'static, str>)) -> Self {
    Self(method, path)
  }
}

impl Display for Route {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.0, self.1)
  }
}

pub type RouteFuture = fn(Request<Body>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>>;
pub type RoutedFunction = (Route, RouteFuture);

pub trait Routable {
  fn routes(&self) -> Vec<RoutedFunction>;
}

pub struct RouteBuilder {
  routes: Vec<RoutedFunction>,
}

impl RouteBuilder {
  pub const fn new() -> Self {
    Self { routes: Vec::new() }
  }

  pub fn add_routes(mut self, routable: &(dyn Routable + 'static)) -> RouteBuilder {
    self.routes.append(&mut routable.routes());
    self
  }

  pub fn not_found_route(self, route: RouteFuture) -> Router {
    Router {
      routes: self.routes,
      not_found_route: route,
    }
  }
}

pub struct Router {
  routes: Vec<RoutedFunction>,
  not_found_route: RouteFuture,
}

impl Router {
  pub async fn route(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let route = Route(req.method().clone(), Cow::Owned(req.uri().path().to_string()));
    info!("Request: {}", &route);

    let func = self
      .routes
      .iter()
      .find(|(r, _)| r == &route)
      .map_or(&self.not_found_route, |(_, func)| func);
    func(req).await
  }

  pub fn builder() -> RouteBuilder {
    RouteBuilder::new()
  }
}

#[macro_export]
macro_rules! route_func {
  ($method:expr, $path:expr, $func:path) => {
    (
      crate::router::Route($method, std::borrow::Cow::Borrowed($path)),
      |req| $func(req).boxed(),
    )
  };
}