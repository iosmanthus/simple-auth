table! {
    accounts (id) {
        id -> Uuid,
        username -> Varchar,
        password -> Varchar,
        salt -> Varchar,
        role -> Int4,
        token -> Nullable<Varchar>,
        token_expire -> Nullable<Timestamp>,
    }
}
