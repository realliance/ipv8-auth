#[macro_use]
extern crate diesel;

use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use tracing::info;

use crate::routes::handle_requests;

pub mod models;
pub mod routes;
pub mod schema;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
  info!("Listening on http://{}", addr);

  let svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_requests)) });

  let server = Server::bind(&addr).serve(svc);

  if let Err(e) = server.await {
    println!("server error: {}", e);
  }
}
