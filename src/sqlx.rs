use sqlx::postgres::PgQueryAs;

#[derive(Clone, sqlx::FromRow)]
pub struct User {
    pub id: Option<i32>,
    pub name: String,
    pub hair_color: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub posts: Option<Vec<String>>,
}

impl crate::Client for sqlx::PgConnection {
    type Error = sqlx::Error;
    type User = User;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        async_std::task::block_on({
            use sqlx::prelude::Connect;
            sqlx::PgConnection::connect(dsn)
        })
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        use sqlx::Executor;
        async_std::task::block_on(self.execute(query)).map(|_| ())
    }

    fn insert_user(&mut self) -> Result<(), Self::Error> {
        async_std::task::block_on({
            sqlx::query("INSERT INTO users (name, hair_color) VALUES ($1, $2)")
                .bind(&"User".to_string())
                .bind(&"hair color".to_string())
                .execute(self)
        })
        .map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::User>, Self::Error> {
        async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT id, name, hair_color, created_at, null as posts FROM users").fetch_all(self)
        })
    }

    fn fetch_first(&mut self) -> Result<Self::User, Self::Error> {
        async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT id, name, hair_color, created_at, null as posts FROM users").fetch_one(self)
        })
    }

    fn fetch_last(&mut self) -> Result<Self::User, Self::Error> {
        let results = async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT id, name, hair_color, created_at, null as posts FROM users").fetch_all(self)
        })?;

        Ok(results[9_999].clone())
    }

    fn one_relation(&mut self) -> Result<(Self::User, Vec<String>), Self::Error> {
        let query = r#"
select u.*, array_agg(p.title) as posts
    from users u
    join posts p on p.author = u.id
    where u.id = $1
    group by u.id, u.name, u.hair_color, u.created_at
"#;
        let user = async_std::task::block_on({
            sqlx::query_as::<_, User>(query).bind(42).fetch_one(self)
        })?;
        let posts = user.posts.clone().unwrap_or_default();

        Ok((user, posts))
    }

    fn all_relations(&mut self) -> Result<Vec<(Self::User, Vec<String>)>, Self::Error> {
         let query = r#"
select u.*, array_agg(p.title) as posts
    from users u
    join posts p on p.author = u.id
    group by u.id, u.name, u.hair_color, u.created_at
"#;
        let users = async_std::task::block_on({
            sqlx::query_as::<_, User>(query).fetch_all(self)
        })?
        .iter()
        .map(|u| {
            let posts = u.posts.clone().unwrap_or_default();
            (u.clone(), posts)
        })
        .collect();

        Ok(users)
    }
}

crate::bench! {sqlx::PgConnection}
