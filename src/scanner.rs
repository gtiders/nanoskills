use crate::models::Config;
use crate::path_utils::normalize_path;
use anyhow::Result;
use ignore::WalkBuilder;
use std::path::Path;

pub struct FileScanner {
    config: Config,
}

impl FileScanner {
    pub fn new(config: Config) -> Self {
        FileScanner { config }
    }

    pub fn scan(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();

        for scan_path in &self.config.scan_paths {
            let path = Path::new(scan_path);
            if !path.exists() {
                continue;
            }

            let mut builder = WalkBuilder::new(path);

            builder
                .hidden(false)
                .git_ignore(true)
                .git_global(true)
                .git_exclude(true)
                .ignore(true)
                .follow_links(false);

            for pattern in &self.config.ignore_patterns {
                builder.add_ignore(pattern);
            }

            for entry in builder.build() {
                match entry {
                    Ok(entry) => {
                        let entry_path = entry.path();
                        if entry_path.is_file() && self.matches_pattern(entry_path) {
                            let normalized = normalize_path(entry_path);
                            files.push(normalized);
                        }
                    }
                    Err(err) => {
                        eprintln!("扫描错误: {}", err);
                    }
                }
            }
        }

        Ok(files)
    }

    fn matches_pattern(&self, path: &Path) -> bool {
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return false,
        };

        for pattern in &self.config.file_patterns {
            if glob_match(pattern, file_name) {
                return true;
            }
        }

        false
    }
}

fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    fn match_helper(pattern: &[char], text: &[char]) -> bool {
        match (pattern.first(), text.first()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some('*'), _) => {
                let pattern_rest = &pattern[1..];
                if pattern_rest.is_empty() {
                    return true;
                }
                for i in 0..=text.len() {
                    if match_helper(pattern_rest, &text[i..]) {
                        return true;
                    }
                }
                false
            }
            (Some('?'), Some(_)) => match_helper(&pattern[1..], &text[1..]),
            (Some(p), Some(t)) if *p == *t => match_helper(&pattern[1..], &text[1..]),
            _ => false,
        }
    }

    match_helper(&pattern_chars, &text_chars)
}

pub fn scan_files(config: &Config) -> Result<Vec<String>> {
    let scanner = FileScanner::new(config.clone());
    scanner.scan()
}
