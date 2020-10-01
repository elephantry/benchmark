use diesel::prelude::*;

#[derive(diesel::Insertable)]
#[table_name = "users"]
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
        id -> Serial,
        name -> VarChar,
        hair_color -> Nullable<VarChar>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    posts {
        id -> Serial,
        title -> Text,
        content -> Text,
        author -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(users, posts);
joinable!(posts -> users (author));

#[derive(Clone, diesel::Queryable, Identifiable)]
pub struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
    created_at: chrono::NaiveDateTime,
}

#[derive(diesel::Insertable)]
#[table_name = "posts"]
pub struct NewPost {
    title: String,
    content: String,
    author: i32,
}

impl NewPost {
    fn new(id: usize, user_id: usize) -> Self {
        Self {
            title: format!("Post number {} for user {}", id, user_id),
            content: "abc".chars().cycle().take(500).collect::<String>(),
            author: user_id as i32,
        }
    }
}

#[derive(Queryable, Associations, Identifiable, Insertable)]
#[belongs_to(User, foreign_key = "author")]
pub struct Post {
    id: i32,
    title: String,
    content: String,
    author: i32,
}

impl crate::Client for diesel::pg::PgConnection {
    type Error = diesel::result::Error;
    type User = User;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        let client = diesel::pg::PgConnection::establish(dsn).unwrap();

        Ok(client)
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        use crate::diesel::connection::SimpleConnection;
        self.batch_execute(query).map(|_| ())
    }

    fn insert_user(&mut self) -> Result<(), Self::Error> {
        diesel::insert_into(users::table)
            .values(&NewUser::new())
            .execute(self)
            .map(|_| ())
    }

    fn insert_users(&mut self, n: usize) -> Result<(), Self::Error> {
        let users = (0..n).map(|_| NewUser::new()).collect::<Vec<_>>();

        diesel::insert_into(users::table)
            .values(&users)
            .execute(self)
            .map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::User>, Self::Error> {
        users::table.load::<Self::User>(self)
    }

    fn fetch_first(&mut self) -> Result<Self::User, Self::Error> {
        let results = users::table.load::<User>(self)?;

        Ok(results.first().unwrap().clone())
    }

    fn fetch_last(&mut self) -> Result<Self::User, Self::Error> {
        let results = users::table.load::<User>(self)?;

        Ok(results[9_999].clone())
    }

    fn one_relation(&mut self) -> Result<(Self::User, Vec<String>), Self::Error> {
        let users = users::table.find(42).first::<User>(self)?;
        let posts = Post::belonging_to(&users).load::<Post>(self)?.iter().map(|x| x.title.clone()).collect();

        Ok((users, posts))
    }

    fn all_relations(&mut self) -> Result<Vec<(Self::User, Vec<String>)>, Self::Error> {
        let users = users::table.load::<User>(self).unwrap();
        let posts = Post::belonging_to(&users)
            .load::<Post>(self)
            .unwrap()
            .grouped_by(&users)
            .iter()
            .map(|u| u.iter().map(|p| p.title.clone()).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        let user_with_posts = users.into_iter().zip(posts).collect();

        Ok(user_with_posts)
    }
}

crate::bench! {diesel::pg::PgConnection}
