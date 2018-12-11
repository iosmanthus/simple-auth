pub mod api;
pub mod credential;
pub mod token;

use rocket::Route;
pub fn engine() -> Vec<Route> {
    routes![self::api::signup, self::api::login, self::api::logout]
}
