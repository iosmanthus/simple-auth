use self::errors::*;
use chrono::{DateTime, NaiveDateTime, Utc};
use crate::db::postgres::PostgresConnection;
use crate::models::accounts::Account;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use std::iter;

pub mod errors {
    error_chain! {
        errors {
            HeaderMissing
                HeaderInvalid
                MethodInvalid(method: String)
                MethodMissing
                TokenMissing
                TokenExpire(token: String)
                DatabaseConnectionError
        }
        links {
            DatabaseExecutionError(crate::models::accounts::errors::Error,
                                   crate::models::accounts::errors::ErrorKind);
        }
    }

}

#[derive(Debug)]
pub struct Token {
    pub token: String,
    pub account: Account,
}

impl Token {
    pub fn verify(token: &str, pg: &PostgresConnection) -> Result<Self> {
        let account = Account::by_token(token, pg)?;
        let exp = account.token_expire.unwrap().timestamp();
        if Utc::now().timestamp() >= exp {
            return Err(ErrorKind::TokenExpire(token.to_string()).into());
        }
        Ok(Token {
            token: token.to_owned(),
            account,
        })
    }

    pub fn generate(len: usize) -> String {
        let mut rng = thread_rng();
        iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(len)
            .collect()
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Token {
    type Error = Error;
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        // Connect to database.
        let pg = request.guard::<PostgresConnection>().map_failure(|_| {
            (
                Status::InternalServerError,
                ErrorKind::DatabaseConnectionError.into(),
            )
        })?;

        // Parse header
        let header = request.headers().get("Authorization").collect::<Vec<_>>();

        // Error code
        let error_code = Status::BadRequest;

        match header.len() {
            0 => Outcome::Failure((error_code, ErrorKind::HeaderMissing.into())),
            1 => {
                let header = header[0].split_whitespace().collect::<Vec<_>>();
                let method = header
                    .get(0)
                    .into_outcome((error_code, ErrorKind::MethodMissing.into()))?;
                match method {
                    &"Bearer" => Token::verify(
                        header
                            .get(1)
                            .into_outcome((error_code, ErrorKind::TokenMissing.into()))?,
                        &pg,
                    )
                    .into_outcome(error_code),

                    _ => Outcome::Failure((
                        error_code,
                        ErrorKind::MethodInvalid(method.to_string()).into(),
                    )),
                }
            }
            _ => Outcome::Failure((error_code, ErrorKind::HeaderInvalid.into())),
        }
    }
}
