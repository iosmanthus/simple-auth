use super::credential::Credential;
use super::token::Token;
use crate::db::postgres::PostgresConnection;
use crate::models::accounts::{Account, NewAccount};
use rocket::http::Status;
use rocket::request::Form;
use rocket::response::status::Custom;
use rocket_contrib::json::Json;

#[derive(Serialize)]
pub struct Response {
    body: String,
    message: String,
}

impl Response {
    pub fn new(body: String, message: String) -> Json<Self> {
        Json(Self { body, message })
    }
}

#[derive(FromForm, Debug)]
pub struct SignupForm {
    name: String,
    password: String,
}

#[get("/login")]
pub fn login(form: Credential, pg: PostgresConnection) -> Custom<Json<Response>> {
    match Account::by_name(&form.username, &pg) {
        Ok(account) => {
            if !account.verify(&form.password) {
                return Custom(
                    Status::Unauthorized,
                    Response::new(String::new(), format!("invalid username or password")),
                );
            }
            if let Some(ref token) = account.token {
                if let Ok(_) = Token::verify(token, &pg) {
                    return Custom(
                        Status::BadRequest,
                        Response::new(
                            String::new(),
                            format!("user: {} has already login", form.username),
                        ),
                    );
                }
            }
            // Token generate
            let token = Token::generate(32);
            if let Err(_) = account.update_token(Some(&token), &pg) {
                return Custom(
                    Status::InternalServerError,
                    Response::new(String::new(), String::from("internal server error")),
                );
            }
            Custom(
                Status::Ok,
                Response::new(token, format!("user: {} login successfully", form.username)),
            )
        }
        Err(_) => Custom(
            Status::Unauthorized,
            Response::new(String::new(), format!("invalid username or password")),
        ),
    }
}

#[post("/signup", data = "<form>")]
pub fn signup(form: Form<SignupForm>, pg: PostgresConnection) -> Custom<Json<Response>> {
    match Account::by_name(&form.name, &pg) {
        Ok(_) => Custom(
            Status::BadRequest,
            Response::new(String::new(), format!("user: {} exists", form.name)),
        ),
        Err(_) => {
            match Account::insert(
                NewAccount::new(
                    form.name.clone(),
                    form.password.clone(),
                    1, /* default permission */
                ),
                &pg,
            ) {
                Ok(_) => Custom(
                    Status::Ok,
                    Response::new(String::new(), format!("user: {} signed up", form.name)),
                ),
                _ => Custom(
                    Status::InternalServerError,
                    Response::new(String::new(), String::from("internal server error")),
                ),
            }
        }
    }
}

#[get("/logout/<username>")]
pub fn logout(username: String, pg: PostgresConnection) -> Custom<Json<Response>> {
    match Account::by_name(&username, &pg) {
        Ok(ref account) if account.token.is_some() => match account.expire_token(&pg) {
            Ok(_) => Custom(
                Status::BadRequest,
                Response::new(
                    String::new(),
                    format!("user: {} logout successfully", username),
                ),
            ),
            _ => Custom(
                Status::InternalServerError,
                Response::new(String::new(), String::from("internal server error")),
            ),
        },
        _ => Custom(
            Status::BadRequest,
            Response::new(String::new(), format!("user: {} doesn't login", username)),
        ),
    }
}
