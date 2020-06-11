use diesel::prelude::*;

#[derive(diesel::Insertable)]
#[table_name = "users"]
pub struct NewUser {
    name: String,
    hair_color: Option<String>,
}

impl NewUser {
    pub fn new(name: String, hair_color: Option<String>) -> Self {
        NewUser { name, hair_color }
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
    created_at: chrono::NaiveDateTime,
}

fn setup() -> Result<diesel::pg::PgConnection, String> {
    let client = diesel::pg::PgConnection::establish(&std::env::var("DATABASE_URL").unwrap())
        .map_err(|e| e.to_string())?;

    client
        .execute("DROP TABLE IF EXISTS users")
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

fn insert_users(client: &diesel::pg::PgConnection, n: usize) -> QueryResult<()> {
    let entries = (0..n)
        .map(|x| NewUser::new(format!("User {}", x), Some(format!("hair color {}", x))))
        .collect::<Vec<_>>();

    diesel::insert_into(users::table)
        .values(&entries)
        .execute(client)?;

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
    insert_users(&client, 1).map_err(|e| e.to_string())?;

    b.iter(|| users::table.first::<User>(&client).unwrap());

    tear_down(&client)
}

#[bench]
fn query_all(b: &mut test::Bencher) -> Result<(), String> {
    let client = setup()?;
    insert_users(&client, 10_000).map_err(|e| e.to_string())?;

    b.iter(|| users::table.load::<User>(&client).unwrap());

    tear_down(&client)
}

#[bench]
fn insert_one(b: &mut test::Bencher) -> Result<(), String> {
    let client = setup()?;

    b.iter(|| {
        // Use a plain insert here to prevent constructing strings inside the
        // benchmark loop
        diesel::insert_into(users::table)
            .values((
                users::name.eq("User 0"),
                users::hair_color.eq("hair color 0"),
            ))
            .execute(&client)
            .unwrap()
    });

    tear_down(&client)
}

#[bench]
fn fetch_first(b: &mut test::Bencher) -> Result<(), String> {
    let client = setup()?;
    insert_users(&client, 10_000).map_err(|e| e.to_string())?;

    b.iter(|| users::table.limit(1).load::<User>(&client).unwrap());

    tear_down(&client)
}

#[bench]
fn fetch_last(b: &mut test::Bencher) -> Result<(), String> {
    let client = setup()?;
    insert_users(&client, 10_000).map_err(|e| e.to_string())?;

    b.iter(|| users::table.offset(9_999).first::<User>(&client).unwrap());

    tear_down(&client)
}

    b.iter(|| {
        let _ = users::table.load::<User>(&client).unwrap()[9_999];
    });

    tear_down(&client)
}
