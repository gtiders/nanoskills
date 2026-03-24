use crate::domain::Config;
use anyhow::Result;
use ignore::{
    WalkBuilder, WalkParallel, WalkState,
    overrides::{Override, OverrideBuilder},
};
use rust_i18n::t;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::path_utils::normalize_path;

const NUL_SNIFF_SIZE: usize = 512;

/// High-performance parallel file scanner used by `sync`.
struct FileScanner<'a> {
    config: &'a Config,
}

impl<'a> FileScanner<'a> {
    fn new(config: &'a Config) -> Self {
        Self { config }
    }

    fn scan(&self) -> Result<Vec<String>> {
        let files = Arc::new(Mutex::new(Vec::new()));
        let max_size = self.config.max_file_size;

        for scan_path in &self.config.scan_paths {
            let root = Path::new(scan_path);
            if !root.exists() {
                eprintln!("{}", t!("cli.scan_path_missing", path = scan_path));
                continue;
            }

            let walker = build_walker(root, &self.config.ignore_patterns)?;
            let shared_files = Arc::clone(&files);

            walker.run(|| {
                let mut collector = LocalCollector::new(Arc::clone(&shared_files));

                Box::new(move |entry| {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            if path.is_file() && is_safe_text_file(path, max_size) {
                                collector.push(normalize_path(path));
                            }
                        }
                        Err(error) => {
                            eprintln!("{}", t!("cli.scan_error", reason = error));
                        }
                    }

                    WalkState::Continue
                })
            });
        }

        Ok(take_scanned_files(files))
    }
}

struct LocalCollector {
    shared: Arc<Mutex<Vec<String>>>,
    local: Vec<String>,
}

impl LocalCollector {
    const FLUSH_THRESHOLD: usize = 256;

    fn new(shared: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            shared,
            local: Vec::with_capacity(Self::FLUSH_THRESHOLD),
        }
    }

    fn push(&mut self, path: String) {
        self.local.push(path);
        if self.local.len() >= Self::FLUSH_THRESHOLD {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if self.local.is_empty() {
            return;
        }

        if let Ok(mut shared) = self.shared.lock() {
            shared.append(&mut self.local);
        } else {
            self.local.clear();
        }
    }
}

impl Drop for LocalCollector {
    fn drop(&mut self) {
        self.flush();
    }
}

fn build_walker(root: &Path, ignore_patterns: &[String]) -> Result<WalkParallel> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .follow_links(false);

    if let Some(overrides) = build_overrides(root, ignore_patterns)? {
        builder.overrides(overrides);
    }

    Ok(builder.build_parallel())
}

fn build_overrides(root: &Path, ignore_patterns: &[String]) -> Result<Option<Override>> {
    if ignore_patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = OverrideBuilder::new(root);
    let mut has_valid_pattern = false;

    for pattern in ignore_patterns.iter().map(String::as_str).map(str::trim) {
        if pattern.is_empty() {
            continue;
        }

        has_valid_pattern = true;
        let override_pattern = pattern
            .strip_prefix('!')
            .map(|raw| format!("!{raw}"))
            .unwrap_or_else(|| format!("!{pattern}"));

        if let Err(error) = builder.add(&override_pattern) {
            eprintln!(
                "{}",
                t!(
                    "cli.invalid_ignore_pattern",
                    pattern = pattern,
                    reason = error
                )
            );
        }
    }

    if !has_valid_pattern {
        return Ok(None);
    }

    builder.build().map(Some).map_err(Into::into)
}

fn take_scanned_files(files: Arc<Mutex<Vec<String>>>) -> Vec<String> {
    match Arc::try_unwrap(files) {
        Ok(mutex) => match mutex.into_inner() {
            Ok(files) => files,
            Err(poisoned) => poisoned.into_inner(),
        },
        Err(shared) => shared.lock().map(|files| files.clone()).unwrap_or_default(),
    }
}

#[inline]
fn passes_size_limit(length: u64, max_size: u64) -> bool {
    length != 0 && length <= max_size
}

#[inline]
fn sniff_contains_nul(sniffed: &[u8], file_len: u64) -> bool {
    let sniff_len = (file_len as usize).min(NUL_SNIFF_SIZE).min(sniffed.len());
    sniffed[..sniff_len].contains(&0x00)
}

#[inline]
fn is_safe_text_file(path: &Path, max_size: u64) -> bool {
    let metadata = match path.metadata() {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };

    let length = metadata.len();
    if !passes_size_limit(length, max_size) {
        return false;
    }

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };

    let sniff_size = (length as usize).min(NUL_SNIFF_SIZE);
    let mut buffer = [0u8; NUL_SNIFF_SIZE];

    // 先用 metadata 做 1MB 上限拦截，再只读取前 512 字节做 NUL 嗅探，
    // 这样可以在不扫描整文件的前提下快速过滤二进制文件，维持当前吞吐路径不退化。
    let bytes_read = match file.read(&mut buffer[..sniff_size]) {
        Ok(bytes_read) => bytes_read,
        Err(_) => return false,
    };

    !sniff_contains_nul(&buffer[..bytes_read], length)
}

/// Scan files with the resolved config.
pub(crate) fn scan_files(config: &Config) -> Result<Vec<String>> {
    FileScanner::new(config).scan()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_MB: u64 = 1024 * 1024;

    #[test]
    fn test_size_gate_rejects_zero_and_oversized_files() {
        // 0 字节文件和超过上限的文件都会在 metadata 阶段被拦截，不会进入后续读取流程。
        assert!(!passes_size_limit(0, ONE_MB));
        assert!(!passes_size_limit(ONE_MB + 1, ONE_MB));
    }

    #[test]
    fn test_size_gate_accepts_exact_max_size() {
        // 恰好等于 1MB 的文件仍然允许继续进入 512B 嗅探，这条边界不能被误伤。
        assert!(passes_size_limit(ONE_MB, ONE_MB));
    }

    #[test]
    fn test_nul_sniff_rejects_binary_marker_within_first_512_bytes() {
        let mut bytes = vec![b'a'; 512];
        bytes[128] = 0x00;

        // NUL 出现在前 512 字节内时必须被判定为二进制文件。
        assert!(sniff_contains_nul(&bytes, 512));
    }

    #[test]
    fn test_nul_sniff_ignores_nul_after_sniff_window() {
        let mut bytes = vec![b'a'; 600];
        bytes[513] = 0x00;

        // 核心性能红线：只检查前 512 字节，因此窗口之后的 NUL 不能影响结果。
        assert!(!sniff_contains_nul(&bytes, 600));
    }
}
