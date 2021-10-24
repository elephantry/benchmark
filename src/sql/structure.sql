begin;

drop table if exists posts;
drop table if exists users;

create table users (
    id uuid primary key default gen_random_uuid(),
    name varchar not null,
    hair_color varchar,
    created_at timestamp not null default now()
);

create table posts (
    id uuid primary key default gen_random_uuid(),
    title text not null,
    content text not null,
    author uuid references users(id)
);

with users as (
    insert into users (id, name, hair_color)
        select gen_random_uuid(), concat('User ', id), concat('hair color ', id)
            from generate_series(1, {}) as id
        union
        select '85e11126-a41d-4dce-98f8-731a87685d2c', 'Sanpi', 'Blue'
        returning *
)
insert into posts (title, content, author)
    select concat('Post number ', g.id, ' for user ', u.id),
        'abc',
        u.id
        from generate_series(1, 30) as g(id), users u;

commit;
