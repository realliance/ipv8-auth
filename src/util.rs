use std::env;

#[inline(always)]
pub fn get_server_url() -> String {
  format!(
    "{}:{}",
    env::var("SERVER_URL").expect("Could not find the environment variable SERVER_URL"),
    env::var("SERVER_PORT").expect("Could not find the environment variable SERVER_PORT")
  )
}

#[inline(always)]
pub fn get_db_url() -> String {
  format!(
    "postgres://{}:{}@{}/{}",
    env::var("DATABASE_USER").expect("Could not find the environment variable DATABASE_USER"),
    env::var("DATABASE_PASS").expect("Could not find the environment variable DATABASE_PASS"),
    env::var("DATABASE_URL").expect("Could not find the environment variable DATABASE_URL"),
    env::var("DATABASE_DB").expect("Could not find the environment variable DATABASE_DB"),
  )
}
