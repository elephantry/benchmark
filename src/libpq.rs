pub struct User {
    pub id: i32,
    pub name: String,
    pub hair_color: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

impl User {
    fn from(result: &libpq::Result, x: usize) -> Result<User, String> {
        let result = to_result(result)?;

        let id = String::from_utf8(result.value(x, 0).unwrap().to_vec()).unwrap().parse().unwrap();
        let name = String::from_utf8(result.value(x, 1).unwrap().to_vec()).unwrap();
        let hair_color = if result.is_null(x, 2) {
            None
        } else {
            String::from_utf8(result.value(x, 2).unwrap().to_vec()).ok()
        };
        let created_at = if result.is_null(x, 2) {
            None
        } else {
            let s = String::from_utf8(result.value(x, 1).unwrap().to_vec()).unwrap();
            chrono::NaiveDateTime::parse_from_str(&s, "%F %T").ok()
        };

        let user = User {
            id,
            name,
            hair_color,
            created_at,
        };

        Ok(user)
    }
}

fn to_result(result: &libpq::Result) -> Result<&libpq::Result, String> {
    use libpq::Status::*;

    match result.status() {
        BadResponse | FatalError | NonFatalError =>  Err(result.error_message().unwrap()),
        _ => Ok(result),
    }
}

impl crate::Client for libpq::Connection{
    type Entity = User;
    type Error = String;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        libpq::Connection::new(dsn)
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        let result = libpq::Connection::exec(self, &query);

        to_result(&result).map(|_| ())
    }

    fn insert_x(&mut self, x: usize) -> Result<(), Self::Error> {
        let name = format!("User {}\0", x);
        let hair_color = format!("hair color {}\0", x);

        let result = libpq::Connection::exec_params(
            self,
            "insert into users (name, hair_color) values ($1, $2)",
            &[],
            &[Some(name.as_bytes().to_vec()), Some(hair_color.as_bytes().to_vec())],
            &[libpq::Format::Text, libpq::Format::Text],
            libpq::Format::Text,
        );

        to_result(&result).map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::Entity>, Self::Error> {
        let result = libpq::Connection::exec(self, "select id, name, hair_color, created_at from users");

        let mut users = Vec::new();

        for x in 0..result.ntuples() {
            users.push(User::from(&result, x)?);
        }

        Ok(users)
    }

    fn fetch_first(&mut self) -> Result<Self::Entity, Self::Error> {
        let result = libpq::Connection::exec(self, "select id, name, hair_color, created_at from users");

        User::from(&result, 0)
    }

    fn fetch_last(&mut self) -> Result<Self::Entity, Self::Error> {
        let result = libpq::Connection::exec(self, "select id, name, hair_color, created_at from users");

        User::from(&result, 9_999)
    }
}

crate::bench! {libpq::Connection}
