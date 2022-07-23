use std::env;

#[inline(always)]
pub fn get_server_url() -> String {
  format!(
    "{}:{}",
    env::var("SERVER_URL").expect("Could not find the environment variable SERVER_URL"),
    env::var("SERVER_PORT").expect("Could not find the environment variable SERVER_PORT")
  )
}
