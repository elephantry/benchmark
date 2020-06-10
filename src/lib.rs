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
