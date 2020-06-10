# Rust PostgresSQL clients benchmark

[diesel](https://crates.io/crates/diesel)
· [elephantry](https://crates.io/crates/elephantry)
· [postgres](https://crates.io/crates/postgres)
· [sqlx](https://crates.io/crates/sqlx)

When I started developping *elephantry* I created this benchmark to check my
code performence, maybe there results could interest other people.

## Run

```
psql --command 'create database bench'
DATABASE_URL="postgres://$USER@localhost/bench" ./graph results/graph.png
```

## Results

![](results/graph.png)
