use crate::models::Config;
use crate::path_utils::normalize_path;
use anyhow::Result;
use ignore::WalkBuilder;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const NUL_SNIFF_SIZE: usize = 512;

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
                        if entry_path.is_file() && is_safe_text_file(entry_path, self.config.max_file_size) {
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
}

pub fn is_safe_text_file(path: &Path, max_size: u64) -> bool {
    let metadata = match path.metadata() {
        Ok(m) => m,
        Err(_) => return false,
    };

    if metadata.len() > max_size {
        return false;
    }

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut buffer = [0u8; NUL_SNIFF_SIZE];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return false,
    };

    if buffer[..bytes_read].contains(&0x00) {
        return false;
    }

    true
}

pub fn scan_files(config: &Config) -> Result<Vec<String>> {
    let scanner = FileScanner::new(config.clone());
    scanner.scan()
}
