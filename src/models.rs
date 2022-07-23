use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chrono::NaiveDateTime;
use diesel::result::Error;
use diesel::{ExpressionMethods, Insertable, PgConnection, QueryDsl, RunQueryDsl};
use crate::diesel::associations::HasTable;
use rand_core::OsRng;
use tracing::{debug, warn};
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

pub fn update_game_stage(conn: &PgConnection, user: User, new_stage: i32) -> User {
  diesel::update(users::table.filter(users::id.eq(user.id)))
  .set(users::license_game_stage.eq(new_stage))
  .get_result(conn)
  .unwrap()
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

#[derive(Associations, Insertable, Queryable, Debug)]
#[belongs_to(User)]
#[table_name = "games"]
pub struct Game {
  pub user_id: Uuid,
  pub token: Uuid,
  pub instruction: i32,
  pub contacted_fizz: bool,
  pub contacted_buzz: bool,
  pub contacted_instructions: bool,
}

impl Game {
  pub fn instruction_completed(&self) -> bool {
    let should_contact_fizz = self.instruction % 3 == 0;
    let should_contact_buzz = self.instruction % 5 == 0;
    let should_contact_neither = !should_contact_fizz && !should_contact_buzz;
    should_contact_fizz == self.contacted_fizz 
    && should_contact_buzz == self.contacted_buzz
    && should_contact_neither == self.contacted_instructions
  }
}

pub fn create_game_instruction(conn: &PgConnection, uid: Uuid, inst: u16) -> Result<(User, Game), Error> {
  let game = Game {
    token: Uuid::new_v4(),
    user_id: uid,
    instruction: inst as i32,
    contacted_fizz: false,
    contacted_buzz: false,
    contacted_instructions: false
  };

  debug!("Creating game instruction {:?}", game);

  let user = {
    use crate::schema::games::dsl::*;
    use crate::schema::users::dsl::*;
    let mut user: User = users.filter(id.eq(uid)).first(conn).unwrap();
    let mut instruction_was_complete: bool = false;


    // Check for previous instruction completed successfully
    let game: Result<Game, _> = games.filter(user_id.eq(uid)).first(conn);
    if let Ok(game) = game {
      if game.instruction_completed() {
        debug!("Instruction was complete!");
        instruction_was_complete = true;
        let new_stage = user.license_game_stage + 1;
        user = update_game_stage(conn, user, new_stage);
      }
    }

    // Clear any old instructions
    let e = diesel::delete(games.filter(user_id.eq(uid))).execute(conn);
    if let Ok(num) = e {
      if num > 0 && !instruction_was_complete {
        debug!("Found instruction to delete for user {}", uid);
        if !user.is_licensed() {
          user = reset_license_game_stage(conn, user).unwrap();
        }
      }
    }

    user
  };

  diesel::insert_into(games::table).values(&game).get_result(conn).map(|game: Game| (user, game))
}

// Mutators for Game Instruction

pub fn update_fizz(conn: &PgConnection, uid: Uuid, instruction_token: Uuid) -> Result<(), Error> {
  use crate::schema::games::dsl::*;
  diesel::update(games.filter(user_id.eq(uid)).filter(token.eq(instruction_token)))
    .set(contacted_fizz.eq(true))
    .execute(conn)
    .map(|_| ())
}

pub fn update_buzz(conn: &PgConnection, uid: Uuid, instruction_token: Uuid) -> Result<(), Error> {
  use crate::schema::games::dsl::*;
  diesel::update(games.filter(user_id.eq(uid)).filter(token.eq(instruction_token)))
    .set(contacted_buzz.eq(true))
    .execute(conn)
    .map(|_| ())
}

pub fn update_instructions(conn: &PgConnection, uid: Uuid, instruction_token: Uuid) -> Result<(), Error> {
  use crate::schema::games::dsl::*;
  diesel::update(games.filter(user_id.eq(uid)).filter(token.eq(instruction_token)))
    .set(contacted_instructions.eq(true))
    .execute(conn)
    .map(|_| ())
}
