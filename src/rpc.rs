use std::convert::Infallible;
use std::{str::FromStr, net::SocketAddr};

use capnp::{capability::Promise, Error};
use capnp_rpc::pry;
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use futures::AsyncReadExt;
use ipv8_proto_rust::user_auth::{Server, GetUserParams, GetUserResults, Client};
use tracing::info;
use uuid::Uuid;

use crate::routes::{util::get_user_by_auth, DB};

struct UserAuth;

impl Server for UserAuth {
  fn get_user(&mut self, params: GetUserParams, mut results: GetUserResults) -> Promise<(), Error> {

    let params = pry!(params.get());
    let auth_token = params.get_auth_token();

    if let Err(err) = auth_token {
      return Promise::err(Error::failed(err.to_string()));
    }
    let auth_token = Uuid::from_str(auth_token.unwrap());

    if let Err(err) = auth_token {
      return Promise::err(Error::failed(err.to_string()));
    }
    let auth_token = auth_token.unwrap();

    let db = DB.blocking_lock();

    if let Ok((user, session)) = get_user_by_auth(&db, auth_token) {
      let mut builder = results.get().init_user();
      builder.set_auth_token(&session.token.to_string());
      builder.set_id(&user.id.to_string());
      builder.set_licensed(user.is_licensed());
      builder.set_name(&user.name);
      builder.set_username(&user.username);

      Promise::ok(())
    } else {
      Promise::err(Error::failed("Could not find user with given auth token".to_string()))
    }
  }
}

pub async fn start_rpc_server(addr: SocketAddr) -> Result<Infallible, std::io::Error> {
  tokio::task::LocalSet::new().run_until(async move {
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let client: Client = capnp_rpc::new_client(UserAuth);

    info!("Started UserAuth RPC Server on TCP {}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        stream.set_nodelay(true)?;
        let (reader, writer) = tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
        let network = twoparty::VatNetwork::new(
            reader,
            writer,
            rpc_twoparty_capnp::Side::Server,
            Default::default(),
        );

        let rpc_system =
            RpcSystem::new(Box::new(network), Some(client.clone().client));

        tokio::task::spawn_local(rpc_system);
      }
  }).await
}
