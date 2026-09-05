use anyhow::{Context, Result, anyhow, bail};
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Cursor;
use std::path::{Path, PathBuf};

const REPOSITORY: &str = "gtiders/skillscripts";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub(crate) fn run(check_only: bool, force: bool) -> Result<()> {
    let client = Client::builder()
        .user_agent(concat!("sks/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let release: Release = client
        .get(format!(
            "https://api.github.com/repos/{REPOSITORY}/releases/latest"
        ))
        .send()
        .context("failed to query the latest GitHub release")?
        .error_for_status()
        .context("GitHub did not return a latest release")?
        .json()
        .context("failed to parse the GitHub release response")?;

    let current = env!("CARGO_PKG_VERSION");
    println!("Current version: {current}");
    println!("Latest version: {}", release.tag_name);
    if !force && release.tag_name.trim_start_matches('v') == current {
        println!("Already up to date.");
        return Ok(());
    }
    if check_only {
        println!("Update available.");
        return Ok(());
    }

    let target = release_target();
    let archive_name = format!("sks-{target}{}", archive_suffix());
    let archive = release
        .assets
        .iter()
        .find(|asset| asset.name == archive_name)
        .ok_or_else(|| anyhow!("release has no asset for compiled target {target}"))?;
    let checksums = release
        .assets
        .iter()
        .find(|asset| asset.name == "checksums.txt")
        .ok_or_else(|| anyhow!("release has no checksums.txt asset"))?;

    let archive_bytes = download(&client, &archive.browser_download_url)?;
    let checksum_text = String::from_utf8(download(&client, &checksums.browser_download_url)?)
        .context("checksums.txt is not valid UTF-8")?;
    verify_checksum(&archive_name, &archive_bytes, &checksum_text)?;

    let temp_root = std::env::temp_dir().join(format!("sks-update-{}", std::process::id()));
    if temp_root.exists() {
        fs::remove_dir_all(&temp_root)?;
    }
    fs::create_dir_all(&temp_root)?;
    let result = install_archive(&archive_bytes, &temp_root);
    if result.is_err() {
        let _ = fs::remove_dir_all(&temp_root);
    }
    let extracted = result?;
    let current_exe = std::env::current_exe().context("failed to locate the current executable")?;
    let replacement = temp_root.join(if cfg!(windows) { "sks.exe" } else { "sks" });
    fs::copy(&extracted, &replacement).context("failed to prepare the downloaded executable")?;
    replace_current(&replacement, &current_exe)?;
    println!("Updated sks to {} for {target}.", release.tag_name);
    Ok(())
}

fn download(client: &Client, url: &str) -> Result<Vec<u8>> {
    Ok(client
        .get(url)
        .send()
        .context("failed to download release asset")?
        .error_for_status()
        .context("GitHub rejected the release asset download")?
        .bytes()
        .context("failed to read release asset")?
        .to_vec())
}

fn verify_checksum(name: &str, bytes: &[u8], checksums: &str) -> Result<()> {
    let expected = checksums
        .lines()
        .find_map(|line| {
            line.contains(name)
                .then(|| line.split_whitespace().next())
                .flatten()
        })
        .ok_or_else(|| anyhow!("checksums.txt has no entry for {name}"))?;
    let actual = format!("{:x}", Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(expected) {
        bail!("checksum mismatch for {name}");
    }
    Ok(())
}

fn install_archive(bytes: &[u8], directory: &Path) -> Result<PathBuf> {
    if cfg!(windows) {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
        for index in 0..archive.len() {
            let mut file = archive.by_index(index)?;
            let Some(path) = file.enclosed_name() else {
                continue;
            };
            if path.file_name().and_then(|value| value.to_str()) == Some("sks.exe") {
                let destination = directory.join("sks.exe");
                let mut output = File::create(&destination)?;
                std::io::copy(&mut file, &mut output)?;
                return Ok(destination);
            }
        }
    } else {
        let decoder = flate2::read::GzDecoder::new(Cursor::new(bytes));
        let mut archive = tar::Archive::new(decoder);
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_path_buf();
            if path.file_name().and_then(|value| value.to_str()) == Some("sks") {
                let destination = directory.join("sks");
                entry.unpack(&destination)?;
                return Ok(destination);
            }
        }
    }
    bail!("release archive does not contain the expected sks executable")
}

fn replace_current(replacement: &Path, current: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        let script = format!(
            "ping 127.0.0.1 -n 2 > nul && copy /Y \"{}\" \"{}\" > nul",
            replacement.display(),
            current.display()
        );
        std::process::Command::new("cmd.exe")
            .args(["/C", &script])
            .spawn()
            .context("failed to start the Windows update helper")?;
        println!("Update staged; the helper will replace sks after this process exits.");
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let permissions = fs::metadata(current)?.permissions();
        fs::set_permissions(replacement, permissions)?;
        fs::rename(replacement, current).context("failed to replace the current executable")?;
        Ok(())
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
const fn release_target() -> &'static str {
    "x86_64-unknown-linux-gnu"
}
#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "musl"))]
const fn release_target() -> &'static str {
    "x86_64-unknown-linux-musl"
}
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const fn release_target() -> &'static str {
    "x86_64-apple-darwin"
}
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const fn release_target() -> &'static str {
    "aarch64-apple-darwin"
}
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const fn release_target() -> &'static str {
    "x86_64-pc-windows-msvc"
}
#[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"),
    all(target_os = "linux", target_arch = "x86_64", target_env = "musl"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64")
)))]
const fn release_target() -> &'static str {
    env!("TARGET")
}

const fn archive_suffix() -> &'static str {
    if cfg!(windows) { ".zip" } else { ".tar.gz" }
}
