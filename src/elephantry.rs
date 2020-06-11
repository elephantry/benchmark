use std::collections::HashMap;

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

mod post {
    #[derive(elephantry::Entity)]
    pub struct Entity {
        pub id: Option<i32>,
        pub title: String,
        pub content: String,
        pub author: i32,
    }

    pub struct Model<'a> {
        connection: &'a elephantry::Connection,
    }

    impl<'a> Model<'a> {
        pub fn posts_for_author(
            &self,
            user: &super::user::Entity,
        ) -> elephantry::Result<Vec<Entity>> {
            use elephantry::{Model, Structure};
            let query = "SELECT {projection} FROM {posts} WHERE {posts}.author = $1";

            let projection = Self::create_projection();

            let sql = query
                .replace("{projection}", &projection.to_string())
                .replace(
                    "{posts}",
                    <Self as elephantry::Model>::Structure::relation(),
                );

            Ok(self
                .connection
                .query::<Entity>(&sql, &[&user.id.unwrap()])?
                .collect())
        }
    }

    impl<'a> elephantry::Model<'a> for Model<'a> {
        type Entity = Entity;
        type Structure = Structure;

        fn new(connection: &'a elephantry::Connection) -> Self {
            Self { connection }
        }
    }

    pub struct Structure;

    impl elephantry::Structure for Structure {
        fn relation() -> &'static str {
            "public.posts"
        }

        fn primary_key() -> &'static [&'static str] {
            &["id"]
        }

        fn definition() -> &'static [&'static str] {
            &["id", "title", "content", "author"]
        }
    }
}

fn setup() -> elephantry::Result<elephantry::Pool> {
    let client = elephantry::Pool::new(&std::env::var("DATABASE_URL").unwrap())?;

    client.execute("DROP TABLE IF EXISTS posts")?;
    client.execute("DROP TABLE IF EXISTS users")?;
    client.execute(
        "CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR NOT NULL,
        hair_color VARCHAR,
        created_at TIMESTAMP NOT NULL DEFAULT NOW()
    )",
    )?;
    client.execute(
        "CREATE TABLE posts (
        id SERIAL PRIMARY KEY,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        author INTEGER REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT
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

fn insert_posts(client: &elephantry::Pool, post_count_per_user: i32) -> elephantry::Result<()> {
    let users = client
        .find_all::<user::Model>(None)
        .unwrap()
        .collect::<Vec<user::Entity>>();

    for user in users {
        for x in 0..post_count_per_user {
            client.insert_one::<post::Model>(&post::Entity {
                id: None,
                title: format!("Post number {} for user {}", x, user.id.unwrap()),
                content: "abc".chars().cycle().take(500).collect::<String>(),
                author: user.id.unwrap(),
            })?;
        }
    }
    Ok(())
}

fn tear_down(client: &elephantry::Pool) -> elephantry::Result<()> {
    client.execute("DROP TABLE posts;")?;
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
fn batch_insert(b: &mut test::Bencher) -> elephantry::Result<()> {
    let mut client = setup()?;

    b.iter(|| insert_users(&mut client, 100).unwrap());

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

#[bench]
fn all_relations(b: &mut test::Bencher) -> elephantry::Result<()> {
    let client = setup()?;
    insert_users(&client, 300)?;
    insert_posts(&client, 30)?;

    b.iter(|| {
        let users = client.find_all::<user::Model>(None).unwrap();

        users
            .into_iter()
            .map(|user| {
                let posts = client
                    .model::<post::Model>()
                    .posts_for_author(&user)
                    .unwrap();
                (user, posts)
            })
            .collect::<Vec<(user::Entity, Vec<post::Entity>)>>()
    });

    tear_down(&client)
}

#[bench]
fn one_relation(b: &mut test::Bencher) -> elephantry::Result<()> {
    let client = setup()?;
    insert_users(&client, 300)?;
    insert_posts(&client, 30)?;

    b.iter(|| {
        let mut pk = HashMap::new();
        pk.insert("id", &42 as &_);
        let user = client.find_by_pk::<user::Model>(&pk).unwrap().unwrap();

        let posts = client
            .model::<post::Model>()
            .posts_for_author(&user)
            .unwrap();
        (user, posts)
    });

    tear_down(&client)
}
