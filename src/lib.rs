#![feature(test)]
#![allow(soft_unstable)]
#![allow(dead_code)]

extern crate test;

// "85e11126-a41d-4dce-98f8-731a87685d2c"
pub const UUID: uuid::Uuid = uuid::Uuid::from_u128(177955938094988552825808298658849381676);

pub trait Client: Sized {
    type Error: Sized;
    type User: Sized;
    type Post: Sized;

    /**
     * Creates a new database connection.
     */
    fn create(dsn: &str) -> Result<Self, Self::Error>;

    /**
     * Execute a simple query (used to create and drop table).
     */
    fn exec(&mut self, query: &str) -> Result<(), Self::Error>;

    /**
     * Insert one row in user table.
     */
    fn insert_user(&mut self) -> Result<(), Self::Error>;

    /**
     * Fetch all rows of a table.
     */
    fn fetch_all(&mut self) -> Result<Vec<Self::User>, Self::Error>;

    /**
     * Fetch only the first result of a rows set.
     */
    fn fetch_first(&mut self) -> Result<Self::User, Self::Error>;

    /**
     * Fetch only the last result of a rows set.
     */
    fn fetch_last(&mut self) -> Result<Self::User, Self::Error>;

    fn one_relation(&mut self) -> Result<(Self::User, Vec<Self::Post>), Self::Error>;

    fn all_relations(&mut self) -> Result<Vec<(Self::User, Vec<Self::Post>)>, Self::Error>;

    fn setup(n: usize) -> Result<Self, Self::Error> {
        env_logger::try_init().ok();

        let dsn = std::env::var("DATABASE_URL").unwrap();

        let mut conn = Self::create(&dsn)?;

        conn.exec(&format!(include_str!("sql/structure.sql"), n))?;

        Ok(conn)
    }

    fn tear_down(&mut self) -> Result<(), Self::Error> {
        self.exec("DROP TABLE IF EXISTS posts").map(|_| ())?;
        self.exec("DROP TABLE IF EXISTS users").map(|_| ())?;

        Ok(())
    }

    fn insert_users(&mut self, n: usize) -> Result<(), Self::Error> {
        for _ in 0..n {
            self.insert_user()?;
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

            b.iter(|| client.insert_users(1).unwrap());

            client.tear_down()
        }

        #[bench]
        fn insert_many(b: &mut test::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(0)?;

            b.iter(|| client.insert_users(25).unwrap());

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

        #[bench]
        fn one_relation(b: &mut test::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(300)?;

            b.iter(|| client.one_relation().unwrap());

            client.tear_down()
        }

        #[bench]
        fn all_relations(b: &mut test::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(300)?;

            b.iter(|| client.all_relations().unwrap());

            client.tear_down()
        }
    };
}
