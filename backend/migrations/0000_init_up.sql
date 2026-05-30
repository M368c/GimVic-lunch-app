CREATE TABLE users (
    id uuid primary key default gen_random_uuid(),
    username varchar(255) not null,
    password varchar(255) not null,
    first_name varchar(255) not null,
    last_name varchar(255) not null,
    token varchar(255) not null
);

CREATE TABLE lunch_optouts (
    id uuid primary key default gen_random_uuid(),
    user_id uuid not null REFERENCES users(id),
    date date not null,
    issued_date TIMESTAMPTZ not null,
    is_send boolean default false,
    UNIQUE(user_id, date)
);