-- Your SQL goes here

CREATE extension IF NOT EXISTS "uuid-ossp";


CREATE TABLE accounts (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  username VARCHAR NOT NULL,
  password VARCHAR NOT NULL,
  salt VARCHAR NOT NULL,
  role INT NOT NULL,
  token VARCHAR,
  token_expire timestamp
);
