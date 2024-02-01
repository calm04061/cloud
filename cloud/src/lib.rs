extern crate core;
#[macro_use]
extern crate rbatis;

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
pub mod plugin;
