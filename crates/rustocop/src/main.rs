use std::process;

mod app;
mod catalog;
mod config;
mod cops;
mod engine;
mod model;

fn main() {
    process::exit(app::run(app::args()));
}
