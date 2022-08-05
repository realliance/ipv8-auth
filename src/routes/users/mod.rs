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
