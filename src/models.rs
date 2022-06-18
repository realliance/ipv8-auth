use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chrono::NaiveDateTime;
use diesel::result::Error;
use diesel::{Insertable, PgConnection, RunQueryDsl};
use rand_core::OsRng;
use uuid::Uuid;

use super::schema::{users, sessions};

#[derive(Identifiable, Queryable)]
#[table_name = "users"]
pub struct User {
  pub id: i32,
  pub name: String,
  pub email: String,
  pub password_digest: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
  pub name: &'a str,
  pub email: &'a str,
  pub password_digest: &'a str,
}

pub fn create_user<'a>(conn: &PgConnection, name: &'a str, email: &'a str, password: &'a str) -> Result<User, Error> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();
  let password_digest = argon2.hash_password(password.as_bytes(), &salt).unwrap();
  let new_user = NewUser {
    name,
    email,
    password_digest: &password_digest.to_string(),
  };

  diesel::insert_into(users::table).values(&new_user).get_result(conn)
}

#[derive(Associations, Insertable, Queryable)]
#[belongs_to(User)]
#[table_name = "sessions"]
pub struct Session {
  pub token: Uuid,
  pub user_id: i32,
  pub last_used: NaiveDateTime,
}
