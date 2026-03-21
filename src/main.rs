mod cli;
mod cmd_sync;
mod config;
mod models;
mod parser;
mod path_utils;
mod scanner;
mod ui;

use anyhow::Result;
use std::path::Path;

rust_i18n::i18n!("locales");

fn detect_language() -> String {
    if let Ok(config) = config::resolve_config(Path::new("."))
        && let Some(lang) = config.language
    {
        return lang;
    }

    if let Some(locale) = sys_locale::get_locale() {
        if locale.starts_with("zh") {
            return "zh-CN".to_string();
        }
        if locale.starts_with("en") {
            return "en".to_string();
        }
    }

    "en".to_string()
}

fn main() -> Result<()> {
    let lang = detect_language();
    rust_i18n::set_locale(&lang);
    cli::run()
}
