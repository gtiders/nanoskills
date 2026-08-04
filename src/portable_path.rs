use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use typed_path::{Utf8UnixComponent, Utf8UnixPath, Utf8WindowsPath};

pub(crate) fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .or_else(|| std::env::var_os("USERPROFILE").filter(|value| !value.is_empty()))
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
        .ok_or_else(|| anyhow::anyhow!("could not determine the user home directory"))
}

pub(crate) fn config_dir() -> Result<PathBuf> {
    Ok(home_dir()?.join(".config").join("sks"))
}

pub(crate) fn agent_skills_dir() -> Result<PathBuf> {
    Ok(home_dir()?.join(".agents").join("skills"))
}

pub(crate) fn resolve_unix_relative(base_dir: &Path, value: &str, label: &str) -> Result<PathBuf> {
    validate_unix_relative(value, label)?;
    let unix = Utf8UnixPath::new(value);
    let mut parts = Vec::new();
    for component in unix.components() {
        match component {
            Utf8UnixComponent::Normal(part) => parts.push(part),
            Utf8UnixComponent::ParentDir => {
                if parts.last().is_some_and(|part| *part != "..") {
                    parts.pop();
                } else {
                    parts.push("..");
                }
            }
            Utf8UnixComponent::CurDir => {}
            Utf8UnixComponent::RootDir => unreachable!("absolute paths are rejected"),
        }
    }
    let mut resolved = base_dir.to_path_buf();
    for part in parts {
        resolved.push(part);
    }

    Ok(resolved)
}

fn validate_unix_relative(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{label} must not be empty");
    }
    if value.contains('\\') {
        bail!("{label} must use '/' separators: {value}");
    }

    let unix = Utf8UnixPath::new(value);
    let windows = Utf8WindowsPath::new(value);
    if unix.is_absolute()
        || windows.has_root()
        || windows
            .components()
            .next()
            .is_some_and(|part| matches!(part, typed_path::Utf8WindowsComponent::Prefix(_)))
    {
        bail!("{label} must be a relative Unix-style path: {value}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_platform_specific_paths_on_every_platform() {
        for value in [
            "/tmp/a.py",
            "C:/tools/a.py",
            r"C:\tools\a.py",
            r"\\host\a.py",
        ] {
            assert!(resolve_unix_relative(Path::new("base"), value, "path").is_err());
        }
    }

    #[test]
    fn resolves_and_normalizes_unix_relative_paths() {
        assert_eq!(
            resolve_unix_relative(Path::new("base"), "scripts/../tools/a.py", "path").unwrap(),
            Path::new("base").join("tools").join("a.py")
        );
        assert_eq!(
            resolve_unix_relative(Path::new("base"), "../shared/a.py", "path").unwrap(),
            Path::new("base").join("..").join("shared").join("a.py")
        );
    }
}
