#![feature(test)]
#![allow(soft_unstable)]

extern crate test;

use diesel::prelude::*;

#[derive(diesel::Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    name: &'a str,
    hair_color: Option<&'a str>,
}

impl<'a> NewUser<'a> {
    pub fn new() -> Self {
        NewUser {
            name: "User",
            hair_color: Some("hair color"),
        }
    }
}

diesel::table! {
    users {
        id -> Uuid,
        name -> VarChar,
        hair_color -> Nullable<VarChar>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    posts {
        id -> Uuid,
        title -> Text,
        content -> Text,
        author -> Uuid,
    }
}

allow_tables_to_appear_in_same_query!(users, posts);
joinable!(posts -> users (author));

#[derive(Clone, Queryable, Identifiable)]
pub struct User {
    id: uuid::Uuid,
    name: String,
    hair_color: Option<String>,
    created_at: chrono::NaiveDateTime,
}

#[derive(diesel::Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost {
    title: String,
    content: String,
    author: uuid::Uuid,
}

#[derive(Queryable, Identifiable, Associations)]
#[diesel(table_name = posts)]
#[diesel(belongs_to(User, foreign_key = author))]
pub struct Post {
    id: uuid::Uuid,
    title: String,
    content: String,
    author: uuid::Uuid,
}

struct Connection(diesel::pg::PgConnection);

impl elephantry_benchmark::Client for Connection {
    type Error = diesel::result::Error;
    type User = User;
    type Post = Post;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        use diesel::Connection;

        let client = diesel::pg::PgConnection::establish(dsn).unwrap();

        Ok(Self(client))
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        use diesel::connection::SimpleConnection;
        self.0.batch_execute(query).map(|_| ())
    }

    fn insert_user(&mut self) -> Result<(), Self::Error> {
        diesel::insert_into(users::table)
            .values(&NewUser::new())
            .execute(&mut self.0)
            .map(|_| ())
    }

    fn insert_users(&mut self, n: usize) -> Result<(), Self::Error> {
        let users = (0..n).map(|_| NewUser::new()).collect::<Vec<_>>();

        diesel::insert_into(users::table)
            .values(&users)
            .execute(&mut self.0)
            .map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::User>, Self::Error> {
        users::table.load::<Self::User>(&mut self.0)
    }

    fn fetch_first(&mut self) -> Result<Self::User, Self::Error> {
        let results = users::table.load::<User>(&mut self.0)?;

        Ok(results.first().unwrap().clone())
    }

    fn fetch_last(&mut self) -> Result<Self::User, Self::Error> {
        let results = users::table.load::<User>(&mut self.0)?;

        Ok(results[9_999].clone())
    }

    fn one_relation(&mut self) -> Result<(Self::User, Vec<Self::Post>), Self::Error> {
        let users = users::table
            .find(elephantry_benchmark::UUID)
            .first::<User>(&mut self.0)?;
        let posts = Post::belonging_to(&users)
            .select(posts::all_columns)
            .load::<Post>(&mut self.0)?
            .into_iter()
            .collect();

        Ok((users, posts))
    }

    fn all_relations(&mut self) -> Result<Vec<(Self::User, Vec<Self::Post>)>, Self::Error> {
        let users = users::table.load(&mut self.0)?;
        let posts: Vec<Post> = Post::belonging_to(&users).load(&mut self.0)?;
        let grouped_posts = posts.grouped_by(&users);
        let users_and_posts = users.into_iter().zip(grouped_posts).collect();

        Ok(users_and_posts)
    }
}

elephantry_benchmark::bench! {Connection}
