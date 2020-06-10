struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
    created_at: chrono::NaiveDateTime,
}

fn setup() -> Result<postgres::Client, postgres::Error> {
    let mut client =
        postgres::Client::connect(&std::env::var("DATABASE_URL").unwrap(), postgres::NoTls)?;

    client.execute(
        "CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR NOT NULL,
        hair_color VARCHAR,
        created_at TIMESTAMP NOT NULL DEFAULT NOW()
    )",
        &[],
    )?;

    Ok(client)
}

fn tear_down(client: &mut postgres::Client) -> Result<(), postgres::Error> {
    client.execute("DROP TABLE users;", &[])?;

    Ok(())
}

fn insert(client: &mut postgres::Client, n: usize) -> Result<(), postgres::Error> {
    for x in 0..n {
        client.execute(
            "INSERT INTO users (name, hair_color) VALUES ($1, $2)",
            &[&format!("User {}", x), &format!("hair color {}", x)],
        )?;
    }

    Ok(())
}

#[bench]
fn query_one(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    let mut client = setup()?;
    insert(&mut client, 1)?;

    b.iter(|| {
        let stmt = client.prepare("SELECT * FROM users LIMIT 1").unwrap();
        let rows = client.query(&stmt, &[]).unwrap();
        User {
            id: rows[0].get("id"),
            name: rows[0].get("name"),
            hair_color: rows[0].get("hair_color"),
            created_at: rows[0].get("created_at"),
        }
    });

    tear_down(&mut client)
}

#[bench]
fn query_all(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    let mut client = setup()?;
    insert(&mut client, 10_000)?;

    b.iter(|| {
        client
            .query("SELECT * FROM users", &[])
            .unwrap()
            .iter()
            .map(|row| User {
                id: row.get("id"),
                name: row.get("name"),
                hair_color: row.get("hair_color"),
                created_at: row.get("created_at"),
            })
            .collect::<Vec<_>>()
    });

    tear_down(&mut client)
}

#[bench]
fn insert_one(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    let mut client = setup()?;

    b.iter(|| {
        insert(&mut client, 1).unwrap();
    });

    tear_down(&mut client)
}

#[bench]
fn fetch_first(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    let mut client = setup()?;
    insert(&mut client, 10_000)?;

    b.iter(|| {
        client
            .query("SELECT * FROM users", &[])
            .unwrap()
            .iter()
            .map(|row| User {
                id: row.get("id"),
                name: row.get("name"),
                hair_color: row.get("hair_color"),
                created_at: row.get("created_at"),
            })
            .next()
            .unwrap();
    });

    tear_down(&mut client)
}

#[bench]
fn fetch_last(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    let mut client = setup()?;
    insert(&mut client, 10_000)?;

    b.iter(|| {
        client
            .query("SELECT * FROM users", &[])
            .unwrap()
            .iter()
            .map(|row| User {
                id: row.get("id"),
                name: row.get("name"),
                hair_color: row.get("hair_color"),
                created_at: row.get("created_at"),
            })
            .nth(9_999)
            .unwrap();
    });

    tear_down(&mut client)
}
