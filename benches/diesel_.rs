use criterion::Bencher;
use diesel::prelude::*;

#[derive(diesel::Insertable)]
#[table_name = "users"]
pub struct NewUser {
    name: String,
    hair_color: Option<String>,
}

impl NewUser {
    pub fn new(name: String, hair_color: Option<String>) -> Self {
        NewUser { name, hair_color }
    }
}

table! {
    users {
        id -> Serial,
        name -> VarChar,
        hair_color -> Nullable<VarChar>,
        created_at -> Timestamp,
    }
}

table! {
    posts {
        id -> Serial,
        title -> Text,
        content -> Text,
        author -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(users, posts);
joinable!(posts -> users (author));

#[derive(diesel::Queryable, Identifiable)]
pub struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
    created_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Associations, Identifiable)]
#[belongs_to(User, foreign_key = "author")]
pub struct Post {
    id: i32,
    title: String,
    content: String,
    author: i32,
}

fn setup() -> Result<diesel::pg::PgConnection, String> {
    let client = diesel::pg::PgConnection::establish(&std::env::var("DATABASE_URL").unwrap())
        .map_err(|e| e.to_string())?;

    client
        .execute("DROP TABLE IF EXISTS posts")
        .map_err(|e| e.to_string())?;
    client
        .execute("DROP TABLE IF EXISTS users")
        .map_err(|e| e.to_string())?;
    client
        .execute(
            "CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR NOT NULL,
        hair_color VARCHAR,
        created_at TIMESTAMP NOT NULL DEFAULT NOW()
    )",
        )
        .map_err(|e| e.to_string())?;
    client
        .execute(
            "CREATE TABLE posts (
        id SERIAL PRIMARY KEY,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        author INTEGER REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT
    )",
        )
        .map_err(|e| e.to_string())?;

    Ok(client)
}

fn insert_users(client: &diesel::pg::PgConnection, n: usize) -> QueryResult<()> {
    let entries = (0..n)
        .map(|x| NewUser::new(format!("User {}", x), Some(format!("hair color {}", x))))
        .collect::<Vec<_>>();

    diesel::insert_into(users::table)
        .values(&entries)
        .execute(client)?;

    Ok(())
}

fn insert_posts(client: &PgConnection, post_count_per_user: i32) -> QueryResult<()> {
    let user_ids = users::table.select(users::id).load::<i32>(client)?;

    let posts = user_ids
        .into_iter()
        .flat_map(|user_id| {
            (0..post_count_per_user).map(move |post_number| {
                (
                    posts::title.eq(format!("Post number {} for user {}", post_number, user_id)),
                    posts::content.eq("abc".chars().cycle().take(500).collect::<String>()),
                    posts::author.eq(user_id),
                )
            })
        })
        .collect::<Vec<_>>();

    diesel::insert_into(posts::table)
        .values(&posts)
        .execute(client)?;
    Ok(())
}

fn tear_down(client: &diesel::pg::PgConnection) -> Result<(), String> {
    client
        .execute("DROP TABLE posts")
        .map_err(|e| e.to_string())?;
    client
        .execute("DROP TABLE users;")
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn query_one(b: &mut Bencher) -> Result<(), String> {
    let client = setup()?;
    insert_users(&client, 1).map_err(|e| e.to_string())?;

    b.iter(|| users::table.first::<User>(&client).unwrap());

    tear_down(&client)
}

pub fn query_all(b: &mut Bencher) -> Result<(), String> {
    let client = setup()?;
    insert_users(&client, 10_000).map_err(|e| e.to_string())?;

    b.iter(|| users::table.load::<User>(&client).unwrap());

    tear_down(&client)
}

pub fn insert_one(b: &mut Bencher) -> Result<(), String> {
    let client = setup()?;

    b.iter(|| {
        // Use a plain insert here to prevent constructing strings inside the
        // benchmark loop
        diesel::insert_into(users::table)
            .values((
                users::name.eq("User 0"),
                users::hair_color.eq("hair color 0"),
            ))
            .execute(&client)
            .unwrap()
    });

    tear_down(&client)
}

pub fn batch_insert(b: &mut Bencher) -> Result<(), String> {
    let client = setup()?;

    b.iter(|| insert_users(&client, 100).unwrap());

    tear_down(&client)
}

pub fn fetch_first(b: &mut Bencher) -> Result<(), String> {
    let client = setup()?;
    insert_users(&client, 10_000).map_err(|e| e.to_string())?;

    b.iter(|| users::table.limit(1).load::<User>(&client).unwrap());

    tear_down(&client)
}

pub fn fetch_last(b: &mut Bencher) -> Result<(), String> {
    let client = setup()?;
    insert_users(&client, 10_000).map_err(|e| e.to_string())?;

    b.iter(|| users::table.offset(9_999).first::<User>(&client).unwrap());

    tear_down(&client)
}

pub fn all_relations(b: &mut Bencher) -> Result<(), String> {
    let client = setup()?;
    insert_users(&client, 300).map_err(|e| e.to_string())?;
    insert_posts(&client, 30).map_err(|e| e.to_string())?;

    b.iter(|| {
        let users = users::table.load::<User>(&client).unwrap();
        let posts = Post::belonging_to(&users)
            .load::<Post>(&client)
            .unwrap()
            .grouped_by(&users);

        let user_with_posts: Vec<(User, Vec<Post>)> = users.into_iter().zip(posts).collect();
        user_with_posts
    });

    Ok(())
}

pub fn one_relation(b: &mut Bencher) -> Result<(), String> {
    let client = setup()?;
    insert_users(&client, 300).map_err(|e| e.to_string())?;
    insert_posts(&client, 30).map_err(|e| e.to_string())?;

    b.iter(|| {
        let users = users::table.find(42).first::<User>(&client).unwrap();
        let posts = Post::belonging_to(&users).load::<Post>(&client).unwrap();
        (users, posts)
    });

    Ok(())
}
