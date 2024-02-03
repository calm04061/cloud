extern crate core;
#[macro_use]
extern crate rbatis;
extern crate dotenv_codegen;

mod config;
pub mod database;
mod domain;
mod error;
mod fs;
mod service;
pub mod storage;
pub mod task;
mod util;
pub mod web;
pub mod plugin;
