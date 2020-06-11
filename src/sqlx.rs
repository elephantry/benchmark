use sqlx::postgres::PgQueryAs;

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: Option<i32>,
    pub name: String,
    pub hair_color: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(sqlx::FromRow)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub author: i32,
}

fn setup() -> Result<sqlx::PgConnection, sqlx::Error> {
    let mut client = async_std::task::block_on({
        use sqlx::prelude::Connect;
        sqlx::PgConnection::connect(&std::env::var("DATABASE_URL").unwrap())
    })?;

    async_std::task::block_on(async {
        sqlx::query("DROP TABLE IF EXISTS posts")
            .execute(&mut client)
            .await?;
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
        sqlx::query(
            "CREATE TABLE posts (
        id SERIAL PRIMARY KEY,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        author INTEGER REFERENCES users(id) ON DELETE CASCADE ON UPDATE RESTRICT
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

fn insert_posts(
    client: &mut sqlx::PgConnection,
    post_count_per_user: i32,
) -> Result<(), sqlx::Error> {
    async_std::task::block_on(async {
        let user_ids = sqlx::query_as::<_, (i32,)>("SELECT id FROM users")
            .fetch_all(&mut *client)
            .await?;

        let mut query = String::from("INSERT INTO posts (title, content, author) VALUES");

        let mut counter: i32 = 0;
        for _user_id in &user_ids {
            for _post in 0..post_count_per_user {
                query += &format!(
                    "{} (${}, ${}, ${})",
                    if counter == 0 { "" } else { "," },
                    3 * counter + 1,
                    3 * counter + 2,
                    3 * counter + 3
                );
                counter += 1;
            }
        }

        let mut query = sqlx::query(&query);
        for (user_id,) in &user_ids {
            for post in 0..post_count_per_user {
                query = query
                    .bind(format!("Post number {} for user {}", post, user_id))
                    .bind("abc".chars().cycle().take(500).collect::<String>())
                    .bind(user_id);
            }
        }

        query.execute(client).await?;

        Result::<_, sqlx::Error>::Ok(())
    })?;

    Ok(())
}

fn tear_down(mut client: &mut sqlx::PgConnection) -> Result<(), sqlx::Error> {
    async_std::task::block_on(async {
        sqlx::query("DROP TABLE posts;")
            .execute(&mut client)
            .await?;
        sqlx::query("DROP TABLE users;")
            .execute(&mut client)
            .await?;

        Result::<_, sqlx::Error>::Ok(())
    })?;

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

#[bench]
fn all_relations(b: &mut test::Bencher) -> Result<(), sqlx::Error> {
    use std::collections::HashMap;

    let mut client = setup()?;
    insert_users(&mut client, 300)?;
    insert_posts(&mut client, 30)?;

    b.iter(|| {
        async_std::task::block_on(async {
            let users = sqlx::query_as::<_, User>(
                "SELECT id, name, hair_color, created_at FROM users",
            )
            .fetch_all(&mut client)
            .await?;

            let user_id = users.iter().map(|user| user.id).collect::<Vec<_>>();

            let posts = sqlx::query_as::<_, Post>(
                "SELECT id, title, content, author FROM posts WHERE author = ANY($1)",
            )
            .bind(user_id)
            .fetch_all(&mut client)
            .await?;

            let mut users_with_post = users
                .into_iter()
                .map(|user| (user.id.unwrap(), (user, Vec::new())))
                .collect::<HashMap<_, _>>();

            for post in posts {
                users_with_post.get_mut(&post.author).unwrap().1.push(post);
            }

            let users_with_post = users_with_post
                .into_iter()
                .map(|(_, v)| v)
                .collect::<Vec<_>>();

            Result::<_, sqlx::Error>::Ok(users_with_post)
        })
        .unwrap()
    });

    Ok(())
}

#[bench]
fn one_relation(b: &mut test::Bencher) -> Result<(), sqlx::Error> {
    let mut client = setup()?;
    insert_users(&mut client, 300)?;
    insert_posts(&mut client, 30)?;

    b.iter(|| {
        async_std::task::block_on(async {
            let user = sqlx::query_as::<_, User>(
                "SELECT id, name, hair_color, created_at FROM users WHERE id = $1",
            )
            .bind(42_i32)
            .fetch_one(&mut client)
            .await?;

            let posts = sqlx::query_as::<_, Post>(
                "SELECT id, title, content, author FROM posts WHERE author = $1",
            )
            .bind(user.id)
            .fetch_all(&mut client)
            .await?;
            Result::<_, sqlx::Error>::Ok((user, posts))
        })
        .unwrap()
    });

    Ok(())
}
