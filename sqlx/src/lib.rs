#![feature(test)]
#![allow(soft_unstable)]

extern crate test;

#[derive(Clone, sqlx::FromRow)]
pub struct User {
    pub id: Option<uuid::Uuid>,
    pub name: String,
    pub hair_color: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub posts: Option<Posts>,
}

/* https://github.com/launchbadge/sqlx/issues/298 */
#[derive(Clone, sqlx::Type)]
#[sqlx(type_name = "_posts")]
pub struct Posts(Vec<Post>);

impl Posts {
    fn to_vec(self) -> Vec<Post> {
        self.0
    }
}

#[derive(Clone, sqlx::Type)]
#[sqlx(type_name = "posts")]
pub struct Post {
    pub id: Option<uuid::Uuid>,
    pub title: String,
    pub content: String,
}

struct Connection(sqlx::PgConnection);

impl elephantry_benchmark::Client for Connection {
    type Error = sqlx::Error;
    type User = User;
    type Post = Post;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        async_std::task::block_on(async {
            use sqlx::Connection;
            sqlx::PgConnection::connect(dsn).await.map(Self)
        })
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        use sqlx::Executor;
        async_std::task::block_on(self.0.execute(query)).map(|_| ())
    }

    fn insert_user(&mut self) -> Result<(), Self::Error> {
        async_std::task::block_on({
            sqlx::query("INSERT INTO users (name, hair_color) VALUES ($1, $2)")
                .bind(&"User".to_string())
                .bind(&"hair color".to_string())
                .execute(&mut self.0)
        })
        .map(|_| ())
    }

    fn insert_users(&mut self, n: usize) -> Result<(), Self::Error> {
        let names = vec!["User"; n];
        let colors = vec!["hair color"; n];
        async_std::task::block_on({
            sqlx::query(
                "INSERT INTO users (name, hair_color) select * from unnest($1::text[], $2::text[])",
            )
            .bind(&names)
            .bind(&colors)
            .execute(&mut self.0)
        })
        .map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::User>, Self::Error> {
        async_std::task::block_on({
            sqlx::query_as::<_, User>(
                "SELECT id, name, hair_color, created_at, null as posts FROM users",
            )
            .fetch_all(&mut self.0)
        })
    }

    fn fetch_first(&mut self) -> Result<Self::User, Self::Error> {
        async_std::task::block_on({
            sqlx::query_as::<_, User>(
                "SELECT id, name, hair_color, created_at, null as posts FROM users",
            )
            .fetch_one(&mut self.0)
        })
    }

    fn fetch_last(&mut self) -> Result<Self::User, Self::Error> {
        let results = async_std::task::block_on({
            sqlx::query_as::<_, User>(
                "SELECT id, name, hair_color, created_at, null as posts FROM users",
            )
            .fetch_all(&mut self.0)
        })?;

        Ok(results[9_999].clone())
    }

    fn one_relation(&mut self) -> Result<(Self::User, Vec<Self::Post>), Self::Error> {
        let query = r#"
select u.*, array_agg(p) as posts
    from users u
    join posts p on p.author = u.id
    where u.id = $1
    group by u.id, u.name, u.hair_color, u.created_at
"#;
        let user = async_std::task::block_on({
            sqlx::query_as::<_, User>(query)
                .bind(elephantry_benchmark::UUID)
                .fetch_one(&mut self.0)
        })?;
        let posts = user.posts.clone().map(Posts::to_vec).unwrap();

        Ok((user, posts))
    }

    fn all_relations(&mut self) -> Result<Vec<(Self::User, Vec<Self::Post>)>, Self::Error> {
        let query = r#"
select u.*, array_agg(p) as posts
    from users u
    join posts p on p.author = u.id
    group by u.id, u.name, u.hair_color, u.created_at
"#;
        let users =
            async_std::task::block_on({ sqlx::query_as::<_, User>(query).fetch_all(&mut self.0) })?
                .iter()
                .map(|u| (u.clone(), u.posts.clone().map(Posts::to_vec).unwrap()))
                .collect();

        Ok(users)
    }
}

elephantry_benchmark::bench! {Connection}
