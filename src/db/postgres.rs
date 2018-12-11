use rocket_contrib::databases::diesel::PgConnection;
#[database("pg_db")]
pub struct PostgresConnection(PgConnection);
