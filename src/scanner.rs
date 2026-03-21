use crate::models::Config;
use crate::path_utils::normalize_path;
use anyhow::Result;
use ignore::{WalkBuilder, WalkParallel, WalkState};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};

const NUL_SNIFF_SIZE: usize = 512;

pub struct FileScanner {
    config: Config,
}

impl FileScanner {
    pub fn new(config: Config) -> Self {
        FileScanner { config }
    }

    pub fn scan(&self) -> Result<Vec<String>> {
        let files: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let max_size = self.config.max_file_size;

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

            let walker: WalkParallel = builder.build_parallel();

            let files_clone = Arc::clone(&files);

            walker.run(|| {
                let files = Arc::clone(&files_clone);

                Box::new(move |entry| {
                    match entry {
                        Ok(entry) => {
                            let entry_path = entry.path();
                            if entry_path.is_file() && is_safe_text_file(entry_path, max_size) {
                                let normalized = normalize_path(entry_path);
                                if let Ok(mut files) = files.lock() {
                                    files.push(normalized);
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Scan error: {}", err);
                        }
                    }
                    WalkState::Continue
                })
            });
        }

        let files = Arc::try_unwrap(files)
            .map_err(|_| anyhow::anyhow!("Failed to unwrap Arc"))?
            .into_inner()
            .map_err(|_| anyhow::anyhow!("Failed to get inner Mutex"))?;

        Ok(files)
    }
}

#[inline]
pub fn is_safe_text_file(path: &Path, max_size: u64) -> bool {
    let metadata = match path.metadata() {
        Ok(m) => m,
        Err(_) => return false,
    };

    let len = metadata.len();

    if len == 0 || len > max_size {
        return false;
    }

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let sniff_size = std::cmp::min(len as usize, NUL_SNIFF_SIZE);
    let mut buffer = vec![0u8; sniff_size];

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
