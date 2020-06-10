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

    async_std::task::block_on({
        sqlx::query(
            "CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            hair_color VARCHAR,
            created_at TIMESTAMP NOT NULL DEFAULT NOW()
        )",
        )
        .execute(&mut client)
    })?;

    Ok(client)
}

fn insert(mut client: &mut sqlx::PgConnection, n: usize) -> Result<(), sqlx::Error> {
    for x in 0..n {
        async_std::task::block_on({
            sqlx::query("INSERT INTO users (name, hair_color) VALUES ($1, $2)")
                .bind(&format!("User {}", x))
                .bind(&format!("hair color {}", x))
                .execute(&mut client)
        })?;
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
    insert(&mut client, 1)?;

    b.iter(|| {
        async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT * FROM users LIMIT 1").fetch_one(&mut client)
        })
    });

    tear_down(&mut client)
}

#[bench]
fn query_all(b: &mut test::Bencher) -> Result<(), sqlx::Error> {
    let mut client = setup()?;
    insert(&mut client, 10_000)?;

    b.iter(|| {
        async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT * FROM users").fetch_all(&mut client)
        })
    });

    tear_down(&mut client)
}

#[bench]
fn insert_one(b: &mut test::Bencher) -> Result<(), sqlx::Error> {
    let mut client = setup()?;

    b.iter(|| insert(&mut client, 1));

    tear_down(&mut client)
}

#[bench]
fn fetch_first(b: &mut test::Bencher) -> Result<(), sqlx::Error> {
    let mut client = setup()?;
    insert(&mut client, 10_000)?;

    b.iter(|| {
        async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT * FROM users").fetch_one(&mut client)
        })
    });

    tear_down(&mut client)
}

#[bench]
fn fetch_last(b: &mut test::Bencher) -> Result<(), sqlx::Error> {
    let mut client = setup()?;
    insert(&mut client, 10_000)?;

    b.iter(|| {
        let result = async_std::task::block_on({
            sqlx::query_as::<_, User>("SELECT * FROM users").fetch_all(&mut client)
        })
        .unwrap();
        result.get(9_999);
    });

    tear_down(&mut client)
}
