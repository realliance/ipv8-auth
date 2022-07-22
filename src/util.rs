use std::env;

#[inline(always)]
pub fn get_server_url() -> String {
  format!(
    "{}:{}",
    env::var("SERVER_URL").unwrap(),
    env::var("SERVER_PORT").unwrap()
  )
}
