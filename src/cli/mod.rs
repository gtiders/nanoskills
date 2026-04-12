#![allow(clippy::module_inception)]
mod cli;
mod commands;
mod output;
mod picker;

pub(crate) use cli::run;
