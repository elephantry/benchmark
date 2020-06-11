pub struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
    created_at: chrono::NaiveDateTime,
}

impl User {
    fn from_row(row: &postgres::Row) -> Self {
        User {
            id: row.get("id"),
            name: row.get("name"),
            hair_color: row.get("hair_color"),
            created_at: row.get("created_at"),
        }
    }
}

impl crate::Client for postgres::Client {
    type Entity = User;
    type Error = postgres::Error;

    fn create(dsn: &str) -> Result<Self, Self::Error> {
        postgres::Client::connect(dsn, postgres::NoTls)
    }

    fn exec(&mut self, query: &str) -> Result<(), Self::Error> {
        self.execute(query, &[]).map(|_| ())
    }

    fn tear_down(&mut self) -> Result<(), Self::Error> {
        self.execute("DROP TABLE users;", &[]).map(|_| ())
    }

    fn insert_x(&mut self, x: usize) -> Result<(), Self::Error> {
        self.execute(
            "INSERT INTO users (name, hair_color) VALUES ($1, $2)",
            &[&format!("User {}", x), &format!("hair color {}", x)],
        )
        .map(|_| ())
    }

    fn fetch_all(&mut self) -> Result<Vec<Self::Entity>, Self::Error> {
        let results = self
            .query("SELECT * FROM users", &[])?
            .iter()
            .map(User::from_row)
            .collect::<Vec<_>>();

        Ok(results)
    }

    fn fetch_first(&mut self) -> Result<Self::Entity, Self::Error> {
        let result = self
            .query("SELECT * FROM users", &[])?
            .iter()
            .map(User::from_row)
            .next()
            .unwrap();

        Ok(result)
    }

    fn fetch_last(&mut self) -> Result<Self::Entity, Self::Error> {
        let result = self
            .query("SELECT * FROM users", &[])?
            .iter()
            .map(User::from_row)
            .nth(9_999)
            .unwrap();

        Ok(result)
    }
}

crate::bench! {postgres::Client}
