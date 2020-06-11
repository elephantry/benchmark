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
register_benchmark!(batch_insert);
register_benchmark!(fetch_first);
register_benchmark!(fetch_last);
register_benchmark!(all_relations);
register_benchmark!(one_relation);

fn setup_criteron(sample_size: usize) -> Criterion {
    Criterion::default().sample_size(sample_size)
}

criterion_group! {
    name = large_benches;
    config = setup_criteron(10);
    targets = query_all, fetch_last, all_relations
}

criterion_group! {
    name = normal_benches;
    config = setup_criteron(25);
    targets = batch_insert, one_relation, fetch_first, query_one, insert_one
}


criterion_main!(normal_benches, large_benches);
