use futures::FutureExt;
use hyper::Method;

use self::register::register_user;
use self::user::get_user_by_token;
use crate::route_func;
use crate::router::{Routable, RoutedFunction};
use crate::routes::users::login::login;

pub mod login;
pub mod register;
pub mod user;

pub struct UserRouter;

impl Routable for UserRouter {
  fn routes(&self) -> Vec<RoutedFunction> {
    vec![
      route_func!(Method::POST, "/register", register_user),
      route_func!(Method::POST, "/login", login),
      route_func!(Method::GET, "/user", get_user_by_token),
    ]
  }
}

#[cfg(test)]
mod test {
  use diesel::RunQueryDsl;
  use lazy_static::lazy_static;

  use crate::routes::users::register::UserBody;
  use crate::routes::DB;

  lazy_static! {
    pub static ref VALID_USER: UserBody = UserBody {
      name: "Tester McTester".to_string(),
      username: "tester".to_string(),
      password: "testtesttest".to_string(),
    };
    pub static ref INVALID_USER_BAD_PASSWORD: UserBody = UserBody {
      name: "Tester McTester".to_string(),
      username: "tester".to_string(),
      password: "testtest".to_string(),
    };
  }

  pub async fn before_user_test() {
    use crate::schema::sessions::dsl::sessions;
    use crate::schema::users::dsl::users;

    dotenv::dotenv().ok();
    let conn = DB.lock().await;
    diesel::delete(sessions).execute(&*conn).unwrap();
    diesel::delete(users).execute(&*conn).unwrap();
  }
}
