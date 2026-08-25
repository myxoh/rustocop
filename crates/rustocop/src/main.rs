use std::process;

mod app;
mod config;
mod cops;
mod engine;
mod model;
mod rubocop;

fn main() {
    process::exit(app::run(app::args()));
}
