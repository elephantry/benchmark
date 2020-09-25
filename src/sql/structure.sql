begin;

drop table if exists posts;
drop table if exists users;

create table users (
    id serial primary key,
    name varchar not null,
    hair_color varchar,
    created_at timestamp not null default now()
);

create table posts (
    id serial primary key,
    title text not null,
    content text not null,
    author integer references users(id)
);

with users as (
    insert into users (name, hair_color)
        select concat('User ', id), concat('hair color ', id)
            from generate_series(1, {}) as id
        returning *
)
insert into posts (title, content, author)
    select concat('Post number ', g.id, ' for user ', u.id),
        'abc',
        u.id
        from generate_series(1, 30) as g(id), users u;

commit;
