mod cli;
mod index;
mod models;
mod parser;
mod path_utils;
mod scanner;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
