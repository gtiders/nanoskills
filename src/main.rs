mod app;
mod domain;
mod infra;
mod presentation;

use anyhow::Result;
use rust_i18n::t;
use std::path::Path;

rust_i18n::i18n!("locales");

fn detect_language() -> String {
    if let Some(language) = app::detect_language(Path::new(".")) {
        return language;
    }

    match sys_locale::get_locale().as_deref() {
        Some(locale) if locale.starts_with("zh") => "zh-CN".to_string(),
        Some(locale) if locale.starts_with("en") => "en".to_string(),
        _ => "en".to_string(),
    }
}

fn run() -> Result<()> {
    rust_i18n::set_locale(&detect_language());
    presentation::run()
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", t!("error.operation_failed", reason = error));
        for cause in error.chain().skip(1) {
            eprintln!("{}", t!("error.caused_by", reason = cause));
        }
        std::process::exit(1);
    }
}
