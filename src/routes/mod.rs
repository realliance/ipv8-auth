use std::convert::Infallible;

use diesel::{Connection, PgConnection};
use futures::FutureExt;
use hyper::{Body, Request, Response, StatusCode};
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tracing::debug;

use crate::respond;
use crate::router::Router;
use crate::routes::game::GameRouter;
use crate::routes::health::HealthRouter;
use crate::routes::users::UserRouter;
use crate::util::get_db_url;

pub mod game;
pub mod health;
pub mod users;
pub mod util;

lazy_static! {
  // This connection sometimes closes. Consider a pool method.
  pub static ref DB: Mutex<PgConnection> = Mutex::new(establish_connection());
  pub static ref ROUTER: Router = {
    debug!("Building Router Table...");
    Router::builder()
      .add_routes(&UserRouter)
      .add_routes(&GameRouter)
      .add_routes(&HealthRouter)
      .not_found_route(|req| not_found_route(req).boxed())
  };
}

fn establish_connection() -> PgConnection {
  let database_url = get_db_url();
  debug!("Connecting to Postgresql...");
  let conn = PgConnection::establish(&database_url);
  match conn {
    Ok(conn) => {
      debug!("Connected to DB");
      conn
    },
    Err(err) => panic!("{}", err),
  }
}

pub async fn handle_requests(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  ROUTER.route(req).await
}

async fn not_found_route(_: Request<Body>) -> Result<Response<Body>, Infallible> {
  respond!(StatusCode::NOT_FOUND, "Not found")
}

#[cfg(test)]
mod test {
  use hyper::Method;

  use super::*;

  pub fn build_test_request<'a>(
    verb: Method,
    path: &'a str,
    body: &'a str,
    auth_token: Option<String>,
  ) -> Request<Body> {
    Request::builder()
      .method(verb)
      .uri(path)
      .header("Authorization", auth_token.unwrap_or_default())
      .body(Body::from(body.to_string()))
      .unwrap()
  }

  #[tokio::test]
  async fn can_receive_404() {
    let res = handle_requests(build_test_request(Method::GET, "/fakepath", "", None))
      .await
      .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
  }
}
