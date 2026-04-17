use anyhow::{Context, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use reqwest::blocking::Client;
use std::time::Duration;

use crate::scanner::Platform; // <-- use the unified type

#[derive(Clone)]
pub struct DriveInfo {
    pub path: String,
    pub has_pico: bool,
}

#[cfg(windows)]
pub fn detect_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();
    for letter in b'A'..=b'Z' {
        let drive_path = format!("{}:\\", letter as char);
        if Path::new(&drive_path).exists() {
            let pico_path = Path::new(&drive_path).join("_pico");
            let has_pico = pico_path.exists() && pico_path.is_dir();
            if has_pico {
                drives.push(DriveInfo {
                    path: drive_path,
                    has_pico,
                });
            }
        }
    }
    drives
}

#[cfg(target_os = "macos")]
pub fn detect_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/Volumes") {
        for entry in entries.flatten() {
            if let Ok(path) = entry.path().canonicalize() {
                let path_str = path.display().to_string();
                let pico_path = path.join("_pico");
                let has_pico = pico_path.exists() && pico_path.is_dir();
                if has_pico {
                    drives.push(DriveInfo {
                        path: path_str,
                        has_pico,
                    });
                }
            }
        }
    }
    drives
}

#[cfg(target_os = "linux")]
pub fn detect_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();
    for base in &["/media", "/mnt"] {
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                if let Ok(path) = entry.path().canonicalize() {
                    let path_str = path.display().to_string();
                    let pico_path = path.join("_pico");
                    let has_pico = pico_path.exists() && pico_path.is_dir();
                    if has_pico {
                        drives.push(DriveInfo {
                            path: path_str,
                            has_pico,
                        });
                    }
                }
            }
        }
    }
    drives
}

// Helper functions for cover art
pub fn extract_game_code(path: &Path, platform: Platform) -> Result<Option<String>> {
    match platform {
        Platform::GBA => read_gba_game_code(path),
        _ => Ok(None),
    }
}

fn read_gba_game_code(path: &Path) -> Result<Option<String>> {
    let mut file = File::open(path).context("opening GBA file")?;
    let mut buffer = [0u8; 4];
    file.seek(SeekFrom::Start(0xAC))?;
    if let Err(err) = file.read_exact(&mut buffer) {
        if err.kind() == std::io::ErrorKind::UnexpectedEof {
            return Ok(None);
        }
        return Err(err).context("reading GBA header");
    }
    if buffer.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'_') {
        Ok(Some(String::from_utf8_lossy(&buffer).to_string()))
    } else {
        Ok(None)
    }
}

pub fn get_default_url_templates(platform: Platform) -> Vec<String> {
    match platform {
        Platform::GBA => vec![
            "https://art.gametdb.com/gba/cover/{region}/{id}.png".to_string(),
            "https://art.gametdb.com/gba/cover/{region}/{id}.jpg".to_string(),
        ],
        _ => vec![
            "http://thumbnails.libretro.com/{system}/Named_Boxarts/{name}.png".to_string(),
        ],
    }
}

pub fn libretro_system_name(platform: Platform) -> &'static str {
    match platform {
        Platform::GB => "Nintendo - Game Boy",
        Platform::GBC => "Nintendo - Game Boy Color",
        Platform::GBA => "Nintendo - Game Boy Advance",
    }
}

pub fn create_http_client(timeout_secs: u64) -> Result<Client> {
    Client::builder()
        .user_agent("pico-cover-gb/0.1")
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .context("Building HTTP client")
}

pub fn scan_rom_files(root: &Path) -> Vec<(PathBuf, Platform)> {
    let mut roms = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some(platform) = Platform::from_extension(ext) {
                roms.push((path.to_path_buf(), platform));
            }
        }
    }
    roms
}

pub fn build_output_path(
    _rom_path: &Path, // prefix unused with underscore
    output_base: &Path,
    platform: Platform,
    game_identifier: &str,
    format: &str,
) -> PathBuf {
    let folder = platform.folder_name();
    let filename = format!("{}.{}", game_identifier, format);
    output_base.join("_pico").join("covers").join(folder).join(filename)
}

pub fn sanitize_for_libretro(name: &str) -> String {
    name.replace(' ', "_")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}