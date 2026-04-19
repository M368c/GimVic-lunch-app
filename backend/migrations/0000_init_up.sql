CREATE TABLE users (
    id uuid primary key default gen_random_uuid(),
    username varchar(255) not null,
    password varchar(255) not null,
    first_name varchar(255) not null,
    last_name varchar(255) not null,
    token varchar(255) not null
);

CREATE TABLE lunch_optouts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id),
    date date NOT NULL,
    issued_date TIMESTAMPTZ NOT NULL,
    UNIQUE(user_id, date)
);