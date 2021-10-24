#![feature(test)]
#![allow(soft_unstable)]

extern crate test;

pub struct User {
    id: uuid::Uuid,
    name: String,
    hair_color: Option<String>,
    created_at: chrono::NaiveDateTime,
    posts: Option<Vec<String>>,
}

impl User {
    fn from_row(row: &postgres::Row) -> Self {
        User {
            id: row.get("id"),
            name: row.get("name"),
            hair_color: row.get("hair_color"),
            created_at: row.get("created_at"),
            posts: row.try_get("posts").ok(),
        }
    }
}

struct Connection(postgres::Client);

impl elephantry_benchmark::Client for Connection {
    type Error = postgres::Error;
    type User = User;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        postgres::Client::connect(dsn, postgres::NoTls)
            .map(Self)
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        self.0.batch_execute(query).map(|_| ())
    }

    fn insert_user(&mut self) -> Result<(), Self::Error> {
        self.0.execute(
            "INSERT INTO users (name, hair_color) VALUES ($1, $2)",
            &[&"User".to_string(), &"hair color".to_string()],
        )
        .map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::User>, Self::Error> {
        let results = self.0
            .query("SELECT id, name, hair_color, created_at FROM users", &[])?
            .iter()
            .map(User::from_row)
            .collect::<Vec<_>>();

        Ok(results)
    }

    fn fetch_first(&mut self) -> Result<Self::User, Self::Error> {
        let result = self.0
            .query("SELECT id, name, hair_color, created_at FROM users", &[])?
            .iter()
            .map(User::from_row)
            .next()
            .unwrap();

        Ok(result)
    }

    fn fetch_last(&mut self) -> Result<Self::User, Self::Error> {
        let result = self.0
            .query("SELECT id, name, hair_color, created_at FROM users", &[])?
            .iter()
            .map(User::from_row)
            .nth(9_999)
            .unwrap();

        Ok(result)
    }

    fn one_relation(&mut self) -> Result<(Self::User, Vec<String>), Self::Error> {
            let query = r#"
select u.*, array_agg(p.title)
    from users u
    join posts p on p.author = u.id
    where u.id = $1
    group by u.id, u.name, u.hair_color, u.created_at
"#;

        let user = self.0
            .query(query, &[&elephantry_benchmark::UUID])?
            .iter()
            .map(User::from_row)
            .next()
            .unwrap();
        let posts = user.posts.clone().unwrap_or_default();

        Ok((user, posts))
    }

    fn all_relations(&mut self) -> Result<Vec<(Self::User, Vec<String>)>, Self::Error> {
             let query = r#"
select u.*, array_agg(p.title)
    from users u
    join posts p on p.author = u.id
    group by u.id, u.name, u.hair_color, u.created_at
"#;

        let users = self.0
            .query(query, &[])?
            .iter()
            .map(|x| {
                let user = User::from_row(x);
                let posts = user.posts.clone().unwrap_or_default();

                (user, posts)
            })
            .collect();

        Ok(users)
   }
}

elephantry_benchmark::bench! {Connection}
