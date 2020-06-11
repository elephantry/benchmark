use postgres::Row;

struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
    created_at: chrono::NaiveDateTime,
}

impl User {
    fn from_row(row: &Row) -> Self {
        User {
            id: row.get("id"),
            name: row.get("name"),
            hair_color: row.get("hair_color"),
            created_at: row.get("created_at"),
        }
    }
}

pub struct Post {
    id: i32,
    title: String,
    content: String,
    author: i32,
}

impl Post {
    fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            title: row.get("title"),
            content: row.get("content"),
            author: row.get("author"),
        }
    }
}

fn setup() -> Result<postgres::Client, postgres::Error> {
    let mut client =
        postgres::Client::connect(&std::env::var("DATABASE_URL").unwrap(), postgres::NoTls)?;

    client.execute("DROP TABLE IF EXISTS posts", &[])?;
    client.execute("DROP TABLE IF EXISTS users", &[])?;
    client.execute(
        "CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR NOT NULL,
        hair_color VARCHAR,
        created_at TIMESTAMP NOT NULL DEFAULT NOW()
    )",
        &[],
    )?;
    client.execute(
        "CREATE TABLE posts (
        id SERIAL PRIMARY KEY,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        author INTEGER REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT
    )",
        &[],
    )?;

    Ok(client)
}

fn tear_down(client: &mut postgres::Client) -> Result<(), postgres::Error> {
    client.execute("DROP TABLE posts;", &[])?;
    client.execute("DROP TABLE users;", &[])?;

    Ok(())
}

fn insert_users(client: &mut postgres::Client, n: usize) -> Result<(), postgres::Error> {
    let mut query = String::from("INSERT INTO users (name, hair_color) VALUES");
    let mut params = Vec::with_capacity(2 * n);
    for x in 0..n {
        query += &format!(
            "{} (${}, ${})",
            if x == 0 { "" } else { "," },
            2 * x + 1,
            2 * x + 2,
        );
        params.push(format!("User {}", x));
        params.push(format!("hair color {}", x));
    }
    let params = params.iter().map(|p| p as _).collect::<Vec<_>>();
    client.execute(&query as &str, &params)?;

    Ok(())
}

fn insert_posts(
    client: &mut postgres::Client,
    post_count_per_user: i32,
) -> Result<(), postgres::Error> {
    let user_ids = client
        .query("SELECT id FROM users", &[])?
        .into_iter()
        .map(|row| row.get("id"))
        .collect::<Vec<i32>>();

    let mut query = String::from("INSERT INTO posts (title, content, author) VALUES");

    let mut counter: i32 = 0;
    let mut params = Vec::with_capacity(user_ids.len() * post_count_per_user as usize);
    for user_id in user_ids {
        for post in 0..post_count_per_user {
            query += &format!(
                "{} (${}, ${}, ${})",
                if counter == 0 { "" } else { "," },
                3 * counter + 1,
                3 * counter + 2,
                3 * counter + 3
            );
            counter += 1;
            params.push((
                format!("Post number {} for user {}", post, user_id),
                "abc".chars().cycle().take(1).collect::<String>(),
                user_id,
            ));
        }
    }

    let params = params
        .iter()
        .flat_map(|(title, content, author)| vec![title as _, content as _, author as _])
        .collect::<Vec<_>>();

    client.execute(&query as &str, &params)?;
    Ok(())
}

#[bench]
fn query_one(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    let mut client = setup()?;
    insert_users(&mut client, 1)?;

    b.iter(|| {
        let stmt = client
            .prepare("SELECT id, name, hair_color, created_at FROM users LIMIT 1")
            .unwrap();
        let rows = client.query(&stmt, &[]).unwrap();
        User::from_row(&rows[0])
    });

    tear_down(&mut client)
}

#[bench]
fn query_all(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    let mut client = setup()?;
    insert_users(&mut client, 10_000)?;

    b.iter(|| {
        client
            .query("SELECT id, name, hair_color, created_at FROM users", &[])
            .unwrap()
            .iter()
            .map(User::from_row)
            .collect::<Vec<_>>()
    });

    tear_down(&mut client)
}

#[bench]
fn insert_one(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    let mut client = setup()?;

    b.iter(|| {
        client
            .execute(
                "INSERT INTO users (name, hair_color) VALUES ($1, $2)",
                &[&"User 0", &"hair color 0"],
            )
            .unwrap()
    });

    tear_down(&mut client)
}

#[bench]
fn batch_insert(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    let mut client = setup()?;

    b.iter(|| insert_users(&mut client, 100).unwrap());

    tear_down(&mut client)
}

#[bench]
fn fetch_first(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    let mut client = setup()?;
    insert_users(&mut client, 10_000)?;

    b.iter(|| {
        client
            .query("SELECT id, name, hair_color, created_at FROM users", &[])
            .unwrap()
            .iter()
            .map(User::from_row)
            .next()
            .unwrap()
    });

    tear_down(&mut client)
}

#[bench]
fn fetch_last(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    let mut client = setup()?;
    insert_users(&mut client, 10_000)?;

    b.iter(|| {
        client
            .query("SELECT id, name, hair_color, created_at FROM users", &[])
            .unwrap()
            .iter()
            .map(User::from_row)
            .nth(9_999)
            .unwrap()
    });

    tear_down(&mut client)
}

#[bench]
fn all_relations(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    use std::collections::HashMap;

    let mut client = setup()?;
    insert_users(&mut client, 300)?;
    insert_posts(&mut client, 30)?;

    b.iter(|| {
        let users = client
            .query("SELECT id, name, hair_color, created_at FROM users", &[])
            .unwrap()
            .iter()
            .map(User::from_row)
            .collect::<Vec<_>>();

        let user_id = users.iter().map(|user| user.id).collect::<Vec<_>>();

        let posts = client
            .query(
                "SELECT id, title, content, author FROM posts WHERE author = ANY($1)",
                &[&user_id],
            )
            .unwrap()
            .iter()
            .map(Post::from_row)
            .collect::<Vec<_>>();

        let mut users_with_post = users
            .into_iter()
            .map(|user| (user.id, (user, Vec::new())))
            .collect::<HashMap<_, _>>();

        for post in posts {
            users_with_post.get_mut(&post.author).unwrap().1.push(post);
        }

        let users_with_post = users_with_post
            .into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<_>>();

        users_with_post
    });

    Ok(())
}

#[bench]
fn one_relation(b: &mut test::Bencher) -> Result<(), postgres::Error> {
    let mut client = setup()?;
    insert_users(&mut client, 300)?;
    insert_posts(&mut client, 30)?;

    b.iter(|| {
        let user = client
            .query(
                "SELECT id, name, hair_color, created_at FROM users WHERE id = $1",
                &[&42_i32 as _],
            )
            .unwrap()
            .iter()
            .map(User::from_row)
            .next()
            .unwrap();

        let posts = client
            .query(
                "SELECT id, title, content, author FROM posts WHERE author = $1",
                &[&user.id],
            )
            .unwrap()
            .iter()
            .map(Post::from_row)
            .collect::<Vec<_>>();

        (user, posts)
    });

    Ok(())
}
