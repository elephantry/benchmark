#![feature(test)]
#![allow(soft_unstable)]
#![allow(dead_code)]

#[macro_use]
extern crate diesel;
extern crate test;

mod diesel_;
mod elephantry;
mod postgres;
mod sqlx;

trait Client: Sized {
    type Entity: Sized;
    type Error: Sized;

    /**
     * Creates a new database connection.
     */
    fn create(dsn: &str) -> Result<Self, Self::Error>;

    /**
     * Execute a simple query (used to create and drop table).
     */
    fn exec(&mut self, query: &str) -> Result<(), Self::Error>;

    /**
     * Insert one row. `x` can be used as unique id.
     */
    fn insert_x(&mut self, x: usize) -> Result<(), Self::Error>;

    /**
     * Fetch all rows of a table.
     */
    fn fetch_all(&mut self) -> Result<Vec<Self::Entity>, Self::Error>;

    /**
     * Fetch only the first result of a rows set.
     */
    fn fetch_first(&mut self) -> Result<Self::Entity, Self::Error>;

    /**
     * Fetch only the last result of a rows set.
     */
    fn fetch_last(&mut self) -> Result<Self::Entity, Self::Error>;

    fn setup(n: usize) -> Result<Self, Self::Error> {
        let dsn = std::env::var("DATABASE_URL").unwrap();
        let query = "CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    hair_color VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
)";

        let mut conn = Self::create(&dsn)?;
        conn.exec(query)?;
        conn.insert(n)?;
        Ok(conn)
    }

    fn tear_down(&mut self) -> Result<(), Self::Error> {
        self.exec("DROP TABLE users").map(|_| ())
    }

    fn insert(&mut self, n: usize) -> Result<(), Self::Error> {
        for x in 0..n {
            self.insert_x(x)?;
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! bench {
    ($ty:ty) => {
        use $crate::Client;

        #[bench]
        fn query_one(b: &mut test::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(1)?;

            b.iter(|| client.fetch_all().unwrap());

            client.tear_down()
        }

        #[bench]
        fn query_all(b: &mut test::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(10_000)?;

            b.iter(|| client.fetch_all().unwrap());

            client.tear_down()
        }

        #[bench]
        fn insert_one(b: &mut test::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(0)?;

            b.iter(|| client.insert(1).unwrap());

            client.tear_down()
        }

        #[bench]
        fn fetch_first(b: &mut test::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(10_000)?;

            b.iter(|| client.fetch_first().unwrap());

            client.tear_down()
        }

        #[bench]
        fn fetch_last(b: &mut test::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(10_000)?;

            b.iter(|| client.fetch_last().unwrap());

            client.tear_down()
        }
    };
}
