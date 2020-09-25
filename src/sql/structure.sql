drop table if exists users;

create table users (
    id serial primary key,
    name varchar not null,
    hair_color varchar,
    created_at timestamp not null default now()
);

insert into users (name, hair_color)
    select concat('User ', id) as name, concat('hair color ', id) as hair_color
        from generate_series(0, {}) as id;
