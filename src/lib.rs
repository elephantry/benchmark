#![feature(test)]
#![allow(soft_unstable)]
#![allow(dead_code)]

#[cfg_attr(feature = "diesel", macro_use)]
#[cfg(feature = "diesel")]
extern crate diesel;
extern crate test;

#[cfg(feature = "diesel")]
mod diesel_;
#[cfg(feature "elephantry")]
mod elephantry;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlx")]
mod sqlx;
