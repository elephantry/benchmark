use sqlx::postgres::PgQueryAs;

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: Option<i32>,
    pub name: String,
    pub hair_color: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

fn setup() -> Result<sqlx::PgConnection, sqlx::Error> {
    let mut client = async_std::task::block_on({
        use sqlx::prelude::Connect;
        sqlx::PgConnection::connect(&std::env::var("DATABASE_URL").unwrap())
    })?;

    async_std::task::block_on(async {
        sqlx::query("DROP TABLE IF EXISTS users")
            .execute(&mut client)
            .await?;
        sqlx::query(
            "CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            hair_color VARCHAR,
            created_at TIMESTAMP NOT NULL DEFAULT NOW()
        )",
        )
        .execute(&mut client)
        .await?;
        Result::<_, sqlx::Error>::Ok(())
    })?;

    Ok(client)
}

fn insert_users(mut client: &mut sqlx::PgConnection, n: usize) -> Result<(), sqlx::Error> {
    let mut query = String::from("INSERT INTO users (name, hair_color) VALUES");
    for x in 0..n {
        query += &format!(
            "{} (${}, ${})",
            if x == 0 { "" } else { "," },
            2 * x + 1,
            2 * x + 2,
        );
    }
    let mut query = sqlx::query(&query);

    for x in 0..n {
        query = query
            .bind(format!("User {}", x))
            .bind(format!("hair color {}", x));
    }

    async_std::task::block_on({ query.execute(&mut client) })?;

    Ok(())
}

    Ok(())
}

fn tear_down(mut client: &mut sqlx::PgConnection) -> Result<(), sqlx::Error> {
    async_std::task::block_on({ sqlx::query("DROP TABLE users;").execute(&mut client) })?;

    Ok(())
}

#[bench]
fn query_one(b: &mut test::Bencher) -> Result<(), sqlx::Error> {
    let mut client = setup()?;
    insert_users(&mut client, 1)?;

    b.iter(|| {
        async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT id, name, hair_color, created_at FROM users LIMIT 1")
                .fetch_one(&mut client)
        })
    });

    tear_down(&mut client)
}

#[bench]
fn query_all(b: &mut test::Bencher) -> Result<(), sqlx::Error> {
    let mut client = setup()?;
    insert_users(&mut client, 10_000)?;

    b.iter(|| {
        async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT id, name, hair_color, created_at FROM users")
                .fetch_all(&mut client)
        })
    });

    tear_down(&mut client)
}

#[bench]
fn insert_one(b: &mut test::Bencher) -> Result<(), sqlx::Error> {
    let mut client = setup()?;

    b.iter(|| {
        async_std::task::block_on({
            sqlx::query("INSERT INTO users (name, hair_color) VALUES ($1, $2)")
                .bind("User 1")
                .bind("hair color 1")
                .execute(&mut client)
        })
    });

    tear_down(&mut client)
}

#[bench]
fn batch_insert(b: &mut test::Bencher) -> Result<(), sqlx::Error> {
    let mut client = setup()?;

    b.iter(|| insert_users(&mut client, 100).unwrap());

    tear_down(&mut client)
}

#[bench]
fn fetch_first(b: &mut test::Bencher) -> Result<(), sqlx::Error> {
    let mut client = setup()?;
    insert_users(&mut client, 10_000)?;

    b.iter(|| {
        async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT id, name, hair_color, created_at FROM users LIMIT 1")
                .fetch_one(&mut client)
        })
    });

    tear_down(&mut client)
}

#[bench]
fn fetch_last(b: &mut test::Bencher) -> Result<(), sqlx::Error> {
    let mut client = setup()?;
    insert_users(&mut client, 10_000)?;

    b.iter(|| {
        async_std::task::block_on({
            sqlx::query_as::<_, User>(
                "SELECT id, name, hair_color, created_at FROM users OFFSET 9999 LIMIT 1",
            )
            .fetch_all(&mut client)
        })
        .unwrap()
    });

    tear_down(&mut client)
}
