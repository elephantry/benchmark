use sqlx::postgres::PgQueryAs;

#[derive(Clone, sqlx::FromRow)]
pub struct User {
    pub id: Option<i32>,
    pub name: String,
    pub hair_color: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

impl crate::Client for sqlx::PgConnection {
    type Entity = User;
    type Error = sqlx::Error;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        async_std::task::block_on({
            use sqlx::prelude::Connect;
            sqlx::PgConnection::connect(dsn)
        })
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        async_std::task::block_on(sqlx::query(query).execute(self)).map(|_| ())
    }

    fn tear_down(&mut self) -> Result<(), Self::Error> {
        async_std::task::block_on(sqlx::query("DROP TABLE users;").execute(self))?;

        Ok(())
    }

    fn insert_x(&mut self, x: usize) -> Result<(), Self::Error> {
        let user = format!("User {}", x);
        let hair_color = format!("hair color {}", x);

        async_std::task::block_on({
            sqlx::query("INSERT INTO users (name, hair_color) VALUES ($1, $2)")
                .bind(&user)
                .bind(&hair_color)
                .execute(self)
        })
        .map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::Entity>, Self::Error> {
        async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT id, name, hair_color, created_at FROM users").fetch_all(self)
        })
    }

    fn fetch_first(&mut self) -> Result<Self::Entity, Self::Error> {
        async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT id, name, hair_color, created_at FROM users").fetch_one(self)
        })
    }

    fn fetch_last(&mut self) -> Result<Self::Entity, Self::Error> {
        let results = async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT id, name, hair_color, created_at FROM users").fetch_all(self)
        })?;

        Ok(results[9_999].clone())
    }
}

crate::bench! {sqlx::PgConnection}
