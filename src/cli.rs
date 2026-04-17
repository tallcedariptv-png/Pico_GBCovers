use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::scanner::scan_roms;
use crate::Args;

pub async fn run_cli(args: &Args) -> Result<()> {
    let root = PathBuf::from(&args.root);
    let overwrite = args.overwrite;
    let regions: Vec<String> = args
        .regions
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let threads = args.threads.unwrap_or_else(num_cpus::get);

    println!("Scanning for ROMs in {}", root.display());
    let roms = scan_roms(&root);
    println!("Found {} ROMs", roms.len());

    let semaphore = Arc::new(Semaphore::new(threads));
    let pb = ProgressBar::new(roms.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
            .progress_chars("#>-"),
    );

    let mut handles = vec![];
    for rom in roms {
        let permit = semaphore.clone().acquire_owned().await?;
        let regions = regions.clone();
        let pb = pb.clone();

        let handle = tokio::spawn(async move {
            let result = process_single_rom(rom, &regions, overwrite).await;
            pb.inc(1);
            drop(permit);
            result
        });
        handles.push(handle);
    }

    for handle in handles {
        if let Err(e) = handle.await? {
            eprintln!("Error: {}", e);
        }
    }

    pb.finish_with_message("Done");
    Ok(())
}

async fn process_single_rom(
    rom: crate::scanner::RomInfo,
    regions: &[String],
    overwrite: bool,
) -> Result<()> {
    use crate::converter::convert_cover;
    use crate::downloader::{cover_exists, download_cover};
    use std::path::PathBuf;

    // Determine drive root (e.g., "E:\" from "E:\Roms\GBA\game.gba")
    let drive_root = rom
        .path
        .components()
        .next()
        .map(|c| c.as_os_str())
        .unwrap_or_default();
    let drive_root_path = PathBuf::from(drive_root);

    // Use the same folder structure as the Reddit user: _pico/covers/gba or _pico/covers/gb
    let covers_dir = drive_root_path
        .join("_pico")
        .join("covers")
        .join(rom.platform.folder_name());

    std::fs::create_dir_all(&covers_dir)?;

    // For GBA use the game ID, for GB/GBC use the file stem
    let output_filename = match rom.platform {
        crate::scanner::Platform::GBA => {
            let id = rom.game_id.as_ref().ok_or_else(|| anyhow::anyhow!("No GBA ID"))?;
            format!("{}.bmp", id)
        }
        _ => format!("{}.bmp", rom.file_stem),
    };
    let output_path = covers_dir.join(output_filename);

    if !overwrite && cover_exists(&output_path) {
        println!("Skipping {} (already exists)", output_path.display());
        return Ok(());
    }

    let image_bytes = download_cover(&rom, regions, 15, |msg| println!("{}", msg))?;
    let converted = convert_cover(&image_bytes)?; // <-- only one argument now
    std::fs::write(&output_path, converted)?;
    println!("Saved cover to {}", output_path.display());

    Ok(())
}