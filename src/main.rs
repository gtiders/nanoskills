mod cli;
mod cmd_sync;
mod config;
mod models;
mod parser;
mod path_utils;
mod scanner;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
