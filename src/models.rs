use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chrono::NaiveDateTime;
use diesel::result::Error;
use diesel::{ExpressionMethods, Insertable, PgConnection, QueryDsl, RunQueryDsl};
use rand_core::OsRng;
use uuid::Uuid;

use super::schema::{sessions, users};

#[derive(Identifiable, Insertable, Queryable)]
#[table_name = "users"]
pub struct User {
  pub id: Uuid,
  pub name: String,
  pub username: String,
  pub password_digest: String,
}

pub fn create_user<'a>(conn: &PgConnection, name: String, username: String, password: String) -> Result<User, Error> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();
  let password_digest = argon2.hash_password(password.as_bytes(), &salt).unwrap();
  let new_user = User {
    id: Uuid::new_v4(),
    name,
    username,
    password_digest: password_digest.to_string(),
  };

  diesel::insert_into(users::table).values(&new_user).get_result(conn)
}

#[derive(Associations, Insertable, Queryable)]
#[belongs_to(User)]
#[table_name = "sessions"]
pub struct Session {
  pub token: Uuid,
  pub user_id: Uuid,
  pub last_used: NaiveDateTime,
}

pub fn update_session_last_used(conn: &PgConnection, token: Uuid) -> Result<(), Error> {
  diesel::update(sessions::table.filter(sessions::token.eq(token)))
    .set(sessions::last_used.eq(chrono::Utc::now().naive_utc()))
    .get_result::<Session>(conn)
    .map(|_| ())
}
