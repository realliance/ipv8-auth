#![feature(associated_type_bounds)]

#[macro_use]
extern crate diesel;

use std::convert::Infallible;
use std::net::SocketAddr;

use dotenv::dotenv;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use tracing::info;

use crate::routes::handle_requests;
use crate::util::get_server_url;

pub mod game;
pub mod models;
pub mod router;
pub mod routes;
pub mod schema;
pub mod util;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
  tracing_subscriber::fmt::init();

  if dotenv().is_ok() {
    info!("Loaded variables from .env");
  }
  let addr: SocketAddr = get_server_url()
    .parse()
    .unwrap_or_else(|_| panic!("Failed to parse server address. Found {}", get_server_url()));

  let svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_requests)) });
  let server = Server::bind(&addr).serve(svc);

  info!("Listening on http://{}", addr);
  if let Err(e) = server.await {
    println!("server error: {}", e);
  }
}
