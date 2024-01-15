create table if not exists sessions
(
    id text primary key not null,
    data blob not null,
    expiry_date integer not null
);