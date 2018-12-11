use self::errors::*;
use chrono::{Duration, NaiveDateTime, Utc};
use crate::schema::accounts;
use diesel::prelude::*;
use diesel::PgConnection;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::iter;
use uuid::Uuid;

pub mod errors {
    error_chain!{
        foreign_links {
            DatabaseError(diesel::result::Error);
        }
    }
}

#[derive(Debug, Queryable)]
pub struct Account {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub salt: String,
    pub role: i32,
    pub token: Option<String>,
    pub token_expire: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[table_name = "accounts"]
pub struct NewAccount {
    username: String,
    password: String,
    salt: String,
    role: i32,
}

impl NewAccount {
    // Generate a random number in specified len
    fn salt_generater(len: usize) -> String {
        let mut rng = thread_rng();
        iter::repeat(())
            .map(|_| rng.sample(Alphanumeric))
            .take(len)
            .collect()
    }
    pub fn new(username: String, password: String, role: i32) -> Self {
        let salt = Self::salt_generater(32);
        let password = hex::encode(argon2rs::argon2i_simple(&password, &salt));
        Self {
            username,
            password,
            salt,
            role,
        }
    }
}

impl Account {
    pub fn by_id(id: Uuid, pg: &PgConnection) -> Result<Self> {
        accounts::table.find(id).first(pg).map_err(|e| e.into())
    }
    pub fn by_name(name: &str, pg: &PgConnection) -> Result<Self> {
        accounts::table
            .filter(accounts::username.eq(name))
            .first(pg)
            .map_err(|e| e.into())
    }
    pub fn by_token(token: &str, pg: &PgConnection) -> Result<Self> {
        accounts::table
            .filter(accounts::token.eq(token))
            .first(pg)
            .map_err(|e| e.into())
    }
    pub fn verify(&self, password: &str) -> bool {
        self.password == hex::encode(argon2rs::argon2i_simple(password, &self.salt))
    }
    pub fn update_token(&self, token: Option<&str>, pg: &PgConnection) -> Result<usize> {
        let now = NaiveDateTime::from_timestamp(Utc::now().timestamp(), 0);
        let exp = now.checked_add_signed(Duration::days(1)).unwrap();

        if let Some(token) = token {
            diesel::update(accounts::table.find(self.id))
                .set(accounts::token.eq(token))
                .execute(pg)?;
        }
        diesel::update(accounts::table.find(self.id))
            .set(accounts::token_expire.eq(exp))
            .execute(pg)
            .map_err(|e| e.into())
    }
    pub fn expire_token(&self, pg: &PgConnection) -> Result<usize> {
        let token: Option<String> = None;
        let exp: Option<NaiveDateTime> = None;
        diesel::update(accounts::table.find(self.id))
            .set(accounts::token.eq(token))
            .execute(pg)?;
        diesel::update(accounts::table.find(self.id))
            .set(accounts::token_expire.eq(exp))
            .execute(pg)
            .map_err(|e| e.into())
    }
    pub fn insert(member: NewAccount, pg: &PgConnection) -> Result<Self> {
        diesel::insert_into(accounts::table)
            .values(&member)
            .get_result(pg)
            .map_err(|e| e.into())
    }
}
