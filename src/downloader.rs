use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use std::path::Path;
use crate::scanner::{Platform, RomInfo};

pub fn download_cover(
    rom: &RomInfo,
    _regions: &[String],
    timeout_secs: u64,
    log: impl Fn(String),
) -> Result<Vec<u8>> {
    let client = Client::builder()
        .user_agent("pico-cover-gb/0.1")
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()?;

    let system = match rom.platform {
        Platform::GB => "Nintendo - Game Boy",
        Platform::GBC => "Nintendo - Game Boy Color",
        Platform::GBA => "Nintendo - Game Boy Advance",
    };

    // Use the raw file stem exactly as it appears in the ROM filename
    let name = &rom.file_stem;
    let url = format!(
        "http://thumbnails.libretro.com/{}/Named_Boxarts/{}.png",
        urlencoding::encode(system),
        urlencoding::encode(name)
    );

    log(format!("Trying Libretro: {}", url));
    let resp = client.get(&url).send()?;
    if !resp.status().is_success() {
        return Err(anyhow!("HTTP error {} for {}", resp.status(), url));
    }

    Ok(resp.bytes()?.to_vec())
}

pub fn cover_exists(output_path: &Path) -> bool {
    output_path.exists()
}