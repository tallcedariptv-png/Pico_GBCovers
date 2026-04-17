use clap::Parser;
use pico_cover_gb::{run_cli, run_gui, Args};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.cli {
        run_cli(&args).await?;
    } else {
        run_gui()?;
    }
    Ok(())
}