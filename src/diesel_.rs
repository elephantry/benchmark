use diesel::prelude::*;

#[derive(diesel::Insertable)]
#[table_name = "users"]
pub struct NewUser {
    name: String,
    hair_color: Option<String>,
}

impl NewUser {
    pub fn new(name: String, hair_color: Option<String>) -> Self {
        NewUser {
            name: name,
            hair_color: hair_color,
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

#[derive(diesel::Queryable)]
pub struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
    created_at: diesel::data_types::PgTimestamp,
}

fn setup() -> Result<diesel::pg::PgConnection, String> {
    let client = diesel::pg::PgConnection::establish(&std::env::var("DATABASE_URL").unwrap())
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

    Ok(client)
}

fn insert(client: &diesel::pg::PgConnection, n: usize) -> Result<(), String> {
    for x in 0..n {
        diesel::insert_into(users::table)
            .values(&NewUser::new(
                format!("User {}", x),
                Some(format!("hair color {}", x)),
            ))
            .execute(client)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn tear_down(client: &diesel::pg::PgConnection) -> Result<(), String> {
    client
        .execute("DROP TABLE users;")
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[bench]
fn query_one(b: &mut test::Bencher) -> Result<(), String> {
    let client = setup()?;
    insert(&client, 1)?;

    b.iter(|| users::table.first::<User>(&client).unwrap());

    tear_down(&client)
}

#[bench]
fn query_all(b: &mut test::Bencher) -> Result<(), String> {
    let client = setup()?;
    insert(&client, 10_000)?;

    b.iter(|| {
        users::table.load::<User>(&client).unwrap();
    });

    tear_down(&client)
}

#[bench]
fn insert_one(b: &mut test::Bencher) -> Result<(), String> {
    let mut client = setup()?;

    b.iter(|| {
        insert(&mut client, 1).unwrap();
    });

    tear_down(&client)
}

#[bench]
fn fetch_first(b: &mut test::Bencher) -> Result<(), String> {
    let client = setup()?;
    insert(&client, 10_000)?;

    b.iter(|| {
        let _ = users::table.load::<User>(&client).unwrap()[0];
    });

    tear_down(&client)
}

#[bench]
fn fetch_last(b: &mut test::Bencher) -> Result<(), String> {
    let client = setup()?;
    insert(&client, 10_000)?;

    b.iter(|| {
        let _ = users::table.load::<User>(&client).unwrap()[9_999];
    });

    tear_down(&client)
}
