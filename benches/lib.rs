#![allow(soft_unstable)]
#![allow(dead_code)]

#[cfg_attr(feature = "diesel", macro_use)]
#[cfg(feature = "diesel")]
extern crate diesel;

use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(feature = "diesel")]
mod diesel_;
#[cfg(feature = "elephantry")]
mod elephantry;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlx")]
mod sqlx;

pub trait Client: Sized {
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
        let query = "
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    hair_color VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);";

        let mut conn = Self::create(&dsn)?;
        conn.exec("DROP TABLE IF EXISTS users")?;
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

        pub fn query_one(b: &mut criterion::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(1)?;

            b.iter(|| client.fetch_all().unwrap());

            client.tear_down()
        }

        pub fn query_all(b: &mut criterion::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(10_000)?;

            b.iter(|| client.fetch_all().unwrap());

            client.tear_down()
        }

        pub fn insert_one(b: &mut criterion::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(0)?;

            b.iter(|| client.insert(1).unwrap());

            client.tear_down()
        }

        pub fn fetch_first(b: &mut criterion::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(10_000)?;

            b.iter(|| client.fetch_first().unwrap());

            client.tear_down()
        }

        pub fn fetch_last(b: &mut criterion::Bencher) -> Result<(), <$ty as $crate::Client>::Error> {
            let mut client: $ty = Client::setup(10_000)?;

            b.iter(|| client.fetch_last().unwrap());

            client.tear_down()
        }
    };
}

macro_rules! register_benchmark {
    ($name: ident) => {
        fn $name(c: &mut Criterion) {
            let mut group = c.benchmark_group(stringify!($name));
            #[cfg(feature = "diesel")]
            {
                group.bench_function("diesel", |b| diesel_::$name(b).unwrap());
            }

            #[cfg(feature = "sqlx")]
            {
                group.bench_function("sqlx", |b| sqlx::$name(b).unwrap());
            }

            #[cfg(feature = "elephantry")]
            {
                group.bench_function("elephantry", |b| elephantry::$name(b).unwrap());
            }

            #[cfg(feature = "postgres")]
            {
                group.bench_function("postgres", |b| postgres::$name(b).unwrap());
            }

            group.finish();
        }
    };
}

register_benchmark!(query_one);
register_benchmark!(query_all);
register_benchmark!(insert_one);
register_benchmark!(fetch_first);
register_benchmark!(fetch_last);

fn setup_criteron(sample_size: usize) -> Criterion {
    Criterion::default().sample_size(sample_size)
}

criterion_group! {
    name = large_benches;
    config = setup_criteron(10);
    targets = query_all, fetch_last
}

criterion_group! {
    name = normal_benches;
    config = setup_criteron(25);
    targets = fetch_first, query_one, insert_one
}

criterion_main!(normal_benches, large_benches);
