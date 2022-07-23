use std::convert::Infallible;
use std::env;

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

mod game;
mod users;
mod util;
mod health;

lazy_static! {
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
  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
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
