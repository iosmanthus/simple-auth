use self::errors::*;
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use std::str::FromStr;

pub mod errors {
    error_chain! {
        errors {
            HeaderMissing
            HeaderInvalid
            MethodInvalid(s: String)
            MethodMissing
            CredentialInvalid
            CredentialMissing
        }
        foreign_links {
            DecodeError(base64::DecodeError);
            EncodingErorr(std::string::FromUtf8Error);
        }
    }
}

#[derive(Debug)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

impl FromStr for Credential {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let bytes = base64::decode(s)?;
        let payload = String::from_utf8(bytes)?;
        let form = payload.split(':').collect::<Vec<_>>();
        let username = form
            .get(0)
            .ok_or::<Error>(ErrorKind::CredentialInvalid.into())?
            .to_string();
        let password = form
            .get(1)
            .ok_or::<Error>(ErrorKind::CredentialInvalid.into())?
            .to_string();
        Ok(Credential { username, password })
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Credential {
    type Error = Error;
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let payload = request.headers().get("Authorization").collect::<Vec<_>>();
        let error_code = Status::BadRequest;
        match payload.len() {
            0 => Outcome::Failure((error_code, ErrorKind::HeaderMissing.into())),
            1 => {
                let payload = payload[0].split_whitespace().collect::<Vec<_>>();
                let method = payload
                    .get(0)
                    .into_outcome((error_code, ErrorKind::MethodMissing.into()))?;

                match method {
                    &"Basic" => Credential::from_str(
                        payload
                            .get(1)
                            .into_outcome((error_code, ErrorKind::CredentialMissing.into()))?,
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
