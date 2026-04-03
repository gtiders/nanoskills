use path_clean::PathClean;
use std::path::{Path, PathBuf};

/// Convert a path into a stable absolute Unix-style string for indexing output.
pub(crate) fn normalize_path(path: &Path) -> String {
    let absolute = make_absolute(path);
    let simplified = simplify_windows_path(&absolute);
    to_unix_style(&simplified)
}

fn make_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    std::fs::canonicalize(path).unwrap_or_else(|_| {
        std::env::current_dir()
            .map(|cwd| cwd.join(path).clean())
            .unwrap_or_else(|_| path.to_path_buf())
    })
}

fn simplify_windows_path(path: &Path) -> PathBuf {
    dunce::simplified(path).to_path_buf()
}

fn to_unix_style(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_unix_style() {
        let path = Path::new("folder/subfolder/file.txt");
        let result = to_unix_style(path);
        assert_eq!(result, "folder/subfolder/file.txt");
    }

    #[test]
    fn test_normalize_returns_absolute() {
        let path = Path::new(".");
        let result = normalize_path(path);
        assert!(result.starts_with('/'));
        assert!(!result.contains('\\'));
    }
}
