#![feature(test)]
#![allow(soft_unstable)]

extern crate test;

pub struct User {
    pub id: uuid::Uuid,
    pub name: String,
    pub hair_color: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub posts: Vec<String>,
}

impl User {
    fn from(result: &libpq::Result, x: usize) -> libpq::errors::Result<User> {
        let result = to_result(result)?;

        let id = String::from_utf8(result.value(x, 0).unwrap().to_vec())
            .unwrap()
            .parse()
            .unwrap();
        let name = String::from_utf8(result.value(x, 1).unwrap().to_vec()).unwrap();
        let hair_color = if result.is_null(x, 2) {
            None
        } else {
            String::from_utf8(result.value(x, 2).unwrap().to_vec()).ok()
        };
        let created_at = if result.is_null(x, 3) {
            None
        } else {
            let s = String::from_utf8(result.value(x, 3).unwrap().to_vec()).unwrap();
            chrono::NaiveDateTime::parse_from_str(&s, "%F %T").ok()
        };

        let posts = if result.nfields() >= 5 {
            String::from_utf8(result.value(x, 4).unwrap_or_default().to_vec())
                .unwrap()
                .trim_matches(|c| c == '{' || c == '}')
                .split(",")
                .map(|x| x.trim_matches('"').to_string())
                .collect()
        } else {
            Vec::new()
        };

        let user = User {
            id,
            name,
            hair_color,
            created_at,
            posts,
        };

        Ok(user)
    }
}

fn to_result(result: &libpq::Result) -> libpq::errors::Result<&libpq::Result> {
    use libpq::Status::*;

    match result.status() {
        BadResponse | FatalError | NonFatalError => {
            let error = result
                .error_message()?
                .map(|x| libpq::errors::Error::Backend(x.to_string()))
                .unwrap_or(libpq::errors::Error::Unknow);

            Err(error)
        }
        _ => Ok(result),
    }
}

struct Connection(libpq::Connection);

impl elephantry_benchmark::Client for Connection {
    type Error = libpq::errors::Error;
    type User = User;
    type Post = String;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        libpq::Connection::new(dsn).map(Self)
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        let result = libpq::Connection::exec(&self.0, &query);

        to_result(&result).map(|_| ())
    }

    fn insert_user(&mut self) -> Result<(), Self::Error> {
        let name = "User\0";
        let hair_color = "hair color\0";

        let result = libpq::Connection::exec_params(
            &self.0,
            "insert into users (name, hair_color) values ($1, $2)",
            &[],
            &[
                Some(name.as_bytes()),
                Some(hair_color.as_bytes()),
            ],
            &[libpq::Format::Text, libpq::Format::Text],
            libpq::Format::Text,
        );

        to_result(&result).map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::User>, Self::Error> {
        let result = libpq::Connection::exec(
            &self.0,
            "select id, name, hair_color, created_at from users",
        );

        let mut users = Vec::new();

        for x in 0..result.ntuples() {
            users.push(User::from(&result, x)?);
        }

        Ok(users)
    }

    fn fetch_first(&mut self) -> Result<Self::User, Self::Error> {
        let result = libpq::Connection::exec(
            &self.0,
            "select id, name, hair_color, created_at from users",
        );

        User::from(&result, 0)
    }

    fn fetch_last(&mut self) -> Result<Self::User, Self::Error> {
        let result = libpq::Connection::exec(
            &self.0,
            "select id, name, hair_color, created_at from users",
        );

        User::from(&result, 9_999)
    }

    fn one_relation(&mut self) -> Result<(Self::User, Vec<Self::Post>), Self::Error> {
        let mut id = elephantry_benchmark::UUID.to_string().as_bytes().to_vec();
        id.push(b'\0');

        let result = libpq::Connection::exec_params(
            &self.0,
            "select u.*, array_agg(p.title)
    from users u
    join posts p on p.author = u.id
    where u.id = $1
    group by u.id, u.name, u.hair_color, u.created_at
            ",
            &[libpq::types::UUID.oid],
            &[Some(&id)],
            &[libpq::Format::Text, libpq::Format::Text],
            libpq::Format::Text,
        );
        let user = User::from(&result, 0)?;
        let posts = user.posts.clone();

        Ok((user, posts))
    }

    fn all_relations(&mut self) -> Result<Vec<(Self::User, Vec<Self::Post>)>, Self::Error> {
        let result = libpq::Connection::exec_params(
            &self.0,
            "select u.*, array_agg(p.title)
    from users u
    join posts p on p.author = u.id
    group by u.id, u.name, u.hair_color, u.created_at
            ",
            &[],
            &[],
            &[],
            libpq::Format::Text,
        );

        let mut users = Vec::new();

        for x in 0..result.ntuples() {
            let user = User::from(&result, x)?;
            let posts = user.posts.clone();

            users.push((user, posts));
        }

        Ok(users)
    }
}

elephantry_benchmark::bench! {Connection}
