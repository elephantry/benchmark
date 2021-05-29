#![feature(test)]
#![allow(soft_unstable)]

#[macro_use]
extern crate diesel;
extern crate test;

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

#[derive(Queryable, Associations, Identifiable)]
#[belongs_to(User, foreign_key = "author")]
pub struct Post {
    id: i32,
    author: i32,
    title: String,
}

struct Connection(diesel::pg::PgConnection);

impl elephantry_benchmark::Client for Connection {
    type Error = diesel::result::Error;
    type User = User;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        use diesel::Connection;

        let client = diesel::pg::PgConnection::establish(dsn).unwrap();

        Ok(Self(client))
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        use crate::diesel::connection::SimpleConnection;
        self.0.batch_execute(query).map(|_| ())
    }

    fn insert_user(&mut self) -> Result<(), Self::Error> {
        diesel::insert_into(users::table)
            .values(&NewUser::new())
            .execute(&self.0)
            .map(|_| ())
    }

    fn insert_users(&mut self, n: usize) -> Result<(), Self::Error> {
        let users = (0..n).map(|_| NewUser::new()).collect::<Vec<_>>();

        diesel::insert_into(users::table)
            .values(&users)
            .execute(&self.0)
            .map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::User>, Self::Error> {
        users::table.load::<Self::User>(&self.0)
    }

    fn fetch_first(&mut self) -> Result<Self::User, Self::Error> {
        let results = users::table.load::<User>(&self.0)?;

        Ok(results.first().unwrap().clone())
    }

    fn fetch_last(&mut self) -> Result<Self::User, Self::Error> {
        let results = users::table.load::<User>(&self.0)?;

        Ok(results[9_999].clone())
    }

    fn one_relation(&mut self) -> Result<(Self::User, Vec<String>), Self::Error> {

        let users = users::table.find(42).first::<User>(&self.0)?;
        let posts = Post::belonging_to(&users).select((posts::id, posts::author, posts::title)).load::<Post>(&self.0)?.into_iter().map(|Post{title, ..}| title).collect();

        Ok((users, posts))
    }

    fn all_relations(&mut self) -> Result<Vec<(Self::User, Vec<String>)>, Self::Error> {
        use self::array_agg::array_agg;

        let res = users::table.inner_join(posts::table).group_by((users::id, users::name, users::hair_color, users::created_at))
                                             .select((users::all_columns, array_agg(posts::title)))
                                             .load::<(User, Vec<String>)>(&self.0)?;

        Ok(res)
    }
}

elephantry_benchmark::bench! {Connection}

#[allow(non_camel_case_types)]
mod array_agg {
    use diesel::expression::{
        AppearsOnTable, AsExpression, Expression, SelectableExpression, NonAggregate,
    };
    use diesel::pg::Pg;
    use diesel::query_builder::{AstPass, QueryFragment,};
    use diesel::result::QueryResult;
    use diesel::sql_types::{Array, SingleValue};
    use std::marker::PhantomData;

    #[derive(Debug, Clone, Copy, QueryId)]
    #[doc(hidden)]
    pub struct array_agg_t<From, To> {
        a: From,
        to: PhantomData<To>,
    }

    pub type array_agg<From, To> = array_agg_t<<From as AsExpression<To>>::Expression, To>;

    pub fn array_agg<From, To>(a: From) -> array_agg<From, To>
    where
        From: AsExpression<To>,
        To: SingleValue,
    {
        array_agg_t {
            a: a.as_expression(),
            to: Default::default(),
        }
    }

    impl<From, To> Expression for array_agg_t<From, To>
    where
        for<'a> &'a From: Expression,
    {
        type SqlType = Array<To>;
    }

    impl<From, To> QueryFragment<Pg> for array_agg_t<From, To>
    where
        for<'a> &'a From: QueryFragment<Pg>,
    {
        fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
            out.push_sql("array_agg(");
            QueryFragment::walk_ast(&(&self.a,), out.reborrow())?;
            out.push_sql(")");
            Ok(())
        }
    }

    impl<From, To, QS> SelectableExpression<QS> for array_agg_t<From, To>
    where
        From: SelectableExpression<QS>,
    array_agg_t<From, To>: AppearsOnTable<QS>,
    {
    }

    impl<From, To, QS> AppearsOnTable<QS> for array_agg_t<From, To>
    where
        From: AppearsOnTable<QS>,
    array_agg_t<From, To>: Expression,
    {
    }
    impl<From, To> NonAggregate for array_agg_t<From, To> {}
}
