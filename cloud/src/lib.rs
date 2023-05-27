#[macro_use]
extern crate rbatis;
extern crate core;

mod config;
pub mod database;
mod domain;
mod error;
mod fs;
mod service;
pub(crate) mod storage;
pub mod task;
mod util;
pub mod web;
