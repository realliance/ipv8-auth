use std::env;

#[inline(always)]
pub fn get_server_url() -> String {
  format!(
    "{}:{}",
    env::var("SERVER_URL").expect("Could not find the environment variable SERVER_URL"),
    env::var("SERVER_PORT").expect("Could not find the environment variable SERVER_PORT")
  )
}

pub fn external_url() -> String {
  env::var("EXTERNAL_URL").unwrap_or(get_server_url())
}

#[inline(always)]
pub fn get_db_url() -> String {
  #[cfg(not(test))]
  let uri = env::var("DATABASE_URI").expect("Could not find the environment variable DATABASE_URL");

  #[cfg(test)]
  let uri = env::var("TEST_DATABASE_URI").expect("Could not find the environment variable TEST_DATABASE_URI");

  format!(
    "postgres://{}:{}@{}/{}",
    env::var("DATABASE_USER").expect("Could not find the environment variable DATABASE_USER"),
    env::var("DATABASE_PASS").expect("Could not find the environment variable DATABASE_PASS"),
    uri,
    env::var("DATABASE_DB").expect("Could not find the environment variable DATABASE_DB"),
  )
}
