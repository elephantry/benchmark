mod user {
    #[derive(elephantry::Entity)]
    pub struct Entity {
        pub id: Option<i32>,
        pub name: String,
        pub hair_color: Option<String>,
        pub created_at: Option<chrono::NaiveDateTime>,
    }

    pub struct Model;

    impl<'a> elephantry::Model<'a> for Model {
        type Entity = Entity;
        type Structure = Structure;

        fn new(_: &'a elephantry::Connection) -> Self {
            Self {}
        }
    }

    pub struct Structure;

    impl elephantry::Structure for Structure {
        fn relation() -> &'static str {
            "public.users"
        }

        fn primary_key() -> &'static [&'static str] {
            &["id"]
        }

        fn definition() -> &'static [&'static str] {
            &["id", "name", "hair_color", "created_at"]
        }
    }
}

fn setup() -> elephantry::Result<elephantry::Pool> {
    let client = elephantry::Pool::new(&std::env::var("DATABASE_URL").unwrap())?;
    client.execute("DROP TABLE IF EXISTS users")?;
    client.execute(
        "CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR NOT NULL,
        hair_color VARCHAR,
        created_at TIMESTAMP NOT NULL DEFAULT NOW()
    )",
    )?;

    Ok(client)
}

fn insert_users(client: &elephantry::Pool, n: usize) -> elephantry::Result<()> {
    for x in 0..n {
        client.insert_one::<user::Model>(&user::Entity {
            id: None,
            name: format!("User {}", x),
            hair_color: Some(format!("hair color {}", x)),
            created_at: None,
        })?;
    }

    Ok(())
}

fn tear_down(client: &elephantry::Pool) -> elephantry::Result<()> {
    client.execute("DROP TABLE users;")?;

    Ok(())
}

#[bench]
fn query_one(b: &mut test::Bencher) -> elephantry::Result<()> {
    let client = setup()?;
    insert_users(&client, 1)?;

    b.iter(|| client.find_all::<user::Model>(Some("LIMIT 1")).unwrap());

    tear_down(&client)
}

#[bench]
fn query_all(b: &mut test::Bencher) -> elephantry::Result<()> {
    let client = setup()?;
    insert_users(&client, 10_000)?;

    #[cfg(feature = "pprof")]
    let guard = pprof::ProfilerGuard::new(100).unwrap();

    b.iter(|| {
        client
            .find_all::<user::Model>(None)
            .unwrap()
            .collect::<Vec<user::Entity>>()
    });

    #[cfg(feature = "pprof")]
    if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create("flamegraph.svg").unwrap();
        report.flamegraph(file).unwrap();
    };

    tear_down(&client)
}

#[bench]
fn insert_one(b: &mut test::Bencher) -> elephantry::Result<()> {
    let mut client = setup()?;

    b.iter(|| {
        insert_users(&mut client, 1).unwrap();
    });

    tear_down(&mut client)
}

#[bench]
fn fetch_first(b: &mut test::Bencher) -> elephantry::Result<()> {
    let client = setup()?;
    insert_users(&client, 10_000)?;

    b.iter(|| client.find_all::<user::Model>(None).unwrap().next());

    tear_down(&client)
}

#[bench]
fn fetch_last(b: &mut test::Bencher) -> elephantry::Result<()> {
    let client = setup()?;
    insert_users(&client, 10_000)?;

    b.iter(|| client.find_all::<user::Model>(None).unwrap().get(9_999));

    tear_down(&client)
}

    b.iter(|| {
    });

    tear_down(&client)
}

#[bench]
    let client = setup()?;

    b.iter(|| {
    });

    tear_down(&client)
}
