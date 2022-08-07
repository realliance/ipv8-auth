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
  use crate::routes::DB;

  pub async fn before_user_test() {
    use crate::schema::sessions::dsl::sessions;
    use crate::schema::users::dsl::users;

    dotenv::dotenv().ok();
    let conn = DB.lock().await;
    diesel::delete(sessions).execute(&*conn).unwrap();
    diesel::delete(users).execute(&*conn).unwrap();
  }
}
