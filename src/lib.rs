pub mod cli;
pub mod converter;
pub mod downloader;
pub mod gui;
pub mod scanner;
pub mod utils;

use clap::Parser;

#[derive(Parser, Clone)] // Add Clone here
#[command(name = "PicoCover GB")]
pub struct Args {
    #[arg(long)]
    pub cli: bool,
    #[arg(long, default_value = ".")]
    pub root: String,
    #[arg(long)]
    pub overwrite: bool,
    #[arg(long, default_value = "EN,US,JP,EU")]
    pub regions: String,
    #[arg(long)]
    pub threads: Option<usize>,
    #[arg(long, default_value = "bmp")]
    pub format: String,
}

pub use cli::run_cli;
pub use gui::run_gui;