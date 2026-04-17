use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    GB,
    GBC,
    GBA,
}

impl Platform {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "gb" => Some(Platform::GB),
            "gbc" => Some(Platform::GBC),
            "gba" => Some(Platform::GBA),
            _ => None,
        }
    }

    pub fn folder_name(&self) -> &'static str {
    match self {
        Platform::GB => "gb",
        Platform::GBC => "gbc",
        Platform::GBA => "gba",
    }
}
}

#[derive(Debug, Clone)]
pub struct RomInfo {
    pub path: PathBuf,
    pub platform: Platform,
    pub game_id: Option<String>,
    pub file_stem: String,
}

pub fn scan_roms(root: &Path) -> Vec<RomInfo> {
    let mut roms = Vec::new();
    for entry in WalkDir::new(root).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some(platform) = Platform::from_extension(ext) {
                let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();
                let game_id = if platform == Platform::GBA {
                    extract_gba_game_id(path)
                } else {
                    None
                };
                roms.push(RomInfo {
                    path: path.to_path_buf(),
                    platform,
                    game_id,
                    file_stem,
                });
            }
        }
    }
    roms
}

fn extract_gba_game_id(path: &Path) -> Option<String> {
    use std::fs::File;
    use std::io::SeekFrom;

    let mut file = File::open(path).ok()?;
    let mut buffer = [0u8; 4];
    file.seek(SeekFrom::Start(0xAC)).ok()?;
    file.read_exact(&mut buffer).ok()?;
    if buffer.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_') {
        Some(String::from_utf8_lossy(&buffer).to_string())
    } else {
        None
    }
}