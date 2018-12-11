#![feature(proc_macro_hygiene, decl_macro)]
#![allow(proc_macro_derive_resolution_fallback)]

extern crate hex;
extern crate rand;
#[macro_use]
extern crate rocket;
extern crate argon2rs;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;

#[macro_use]
extern crate error_chain;
extern crate uuid;

#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate jsonwebtoken as jwt;

pub mod auth;
pub mod db;
pub mod models;
pub mod schema;

use self::db::postgres::PostgresConnection;
use rocket::Request;

#[catch(400)]
fn bad_request(_req: &Request) -> String {
    "! Fuck you ! try to attack me? No WAY!".to_string()
}

use self::auth::token::Token;
#[get("/test")]
fn test(token: Token) {
    println!("{:#?}", token);
}

pub fn run() {
    rocket::ignite()
        .attach(PostgresConnection::fairing())
        .register(catchers![bad_request])
        .mount("/api", self::auth::engine())
        .mount("/api", routes![test])
        .launch();
}
