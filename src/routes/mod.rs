use std::convert::Infallible;
use std::env;

use diesel::{Connection, PgConnection};
use dotenv::dotenv;
use hyper::{Body, Method, Request, Response};
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tracing::info;

mod users;

lazy_static! {
  pub static ref DB: Mutex<PgConnection> = Mutex::new(establish_connection());
}

fn establish_connection() -> PgConnection {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub async fn handle_requests(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let (method, uri) = (req.method(), req.uri().path());

  info!("Request: {:?}", (method, uri));

  match (method, uri) {
    (&Method::POST, "/register") => users::register_user(req).await,
    (&Method::POST, "/login") => users::login(req).await,
    _ => Ok(Response::builder().status(404).body(Body::from("Not found")).unwrap()),
  }
}
