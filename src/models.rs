use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chrono::NaiveDateTime;
use diesel::result::Error;
use diesel::{ExpressionMethods, Insertable, PgConnection, QueryDsl, RunQueryDsl};
use rand_core::OsRng;
use uuid::Uuid;

use super::schema::*;

#[derive(Identifiable, Insertable, Queryable, Clone)]
#[table_name = "users"]
pub struct User {
  pub id: Uuid,
  pub name: String,
  pub username: String,
  pub password_digest: String,
  pub license_game_stage: i32,
}

impl User {
  pub fn is_licensed(&self) -> bool {
    self.license_game_stage >= 150
  }
}

pub fn create_user<'a>(
  conn: &PgConnection,
  name: String,
  username: String,
  password: String,
  license_game_stage: i32,
) -> Result<User, Error> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();
  let password_digest = argon2.hash_password(password.as_bytes(), &salt).unwrap();
  let new_user = User {
    id: Uuid::new_v4(),
    name,
    username,
    password_digest: password_digest.to_string(),
    license_game_stage,
  };

  diesel::insert_into(users::table).values(&new_user).get_result(conn)
}

pub fn reset_license_game_stage(conn: &PgConnection, user: User) -> Result<User, Error> {
  diesel::update(users::table.filter(users::id.eq(user.id)))
    .set(users::license_game_stage.eq(0))
    .get_result(conn)
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

#[derive(Associations, Insertable, Queryable)]
#[belongs_to(User)]
#[table_name = "games"]
pub struct Game {
  pub token: Uuid,
  pub user_id: Uuid,
  pub instruction: i32,
}

pub fn create_game_instruction(conn: &PgConnection, id: Uuid, inst: i32) -> Result<Game, Error> {
  let game = Game {
    token: Uuid::new_v4(),
    user_id: id,
    instruction: inst,
  };

  // Clear any old instructions
  {
    use crate::schema::games::dsl::*;
    let _ = diesel::delete(games.filter(user_id.eq(user_id))).execute(conn);
  }

  diesel::insert_into(games::table).values(&game).get_result(conn)
}
