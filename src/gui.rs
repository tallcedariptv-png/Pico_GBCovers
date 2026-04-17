use eframe::{egui, Frame};
use egui::{Context, RichText, ScrollArea};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::scanner::{scan_roms, RomInfo};
use crate::utils::{detect_drives, DriveInfo};

pub fn run_gui() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "PicoCover GB",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(GuiApp::new())
        }),
    )
    .map_err(|e| anyhow::anyhow!("Eframe error: {}", e))?;

    Ok(())
}

#[derive(Debug, Clone)]
enum GuiMessage {
    Log(String),
    Progress(usize, usize),
    Done,
}

struct GuiApp {
    overwrite: bool,
    regions: String,
    output_format: String,
    processing: bool,
    logs: Vec<String>,
    progress: (usize, usize),
    tx: Option<Sender<GuiMessage>>,
    rx: Option<Receiver<GuiMessage>>,
    // Drive selection
    drives: Vec<DriveInfo>,
    selected_drive: usize,
}

impl GuiApp {
    fn new() -> Self {
        let drives = detect_drives();
        let selected_drive = drives.iter().position(|d| d.has_pico).unwrap_or(0);
        Self {
            overwrite: false,
            regions: "EN,US,JP,EU".to_string(),
            output_format: "png".to_string(),
            processing: false,
            logs: Vec::new(),
            progress: (0, 0),
            tx: None,
            rx: None,
            drives,
            selected_drive,
        }
    }

    fn add_log(&mut self, msg: impl Into<String>) {
        self.logs.push(msg.into());
    }

    fn refresh_drives(&mut self) {
        self.drives = detect_drives();
        if self.selected_drive >= self.drives.len() {
            self.selected_drive = 0;
        }
    }

    fn start_processing(&mut self) {
    if self.drives.is_empty() {
        self.add_log("No drives detected. Please insert a micro SD card and refresh.");
        return;
    }

    // Clone all data that will be moved into the thread
    let drive_path = self.drives[self.selected_drive].path.clone(); // owned String
    let overwrite = self.overwrite;
    let regions: Vec<String> = self
        .regions
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let output_format = self.output_format.clone();

    self.processing = true;
    self.logs.clear();
    self.progress = (0, 0);

    let (tx, rx) = channel();
    self.tx = Some(tx.clone());
    self.rx = Some(rx);

    thread::spawn(move || {
        let root = PathBuf::from(&drive_path);
        let roms = scan_roms(&root);
        let total = roms.len();
        tx.send(GuiMessage::Log(format!("Scanning drive: {}", drive_path)))
            .ok();
        tx.send(GuiMessage::Log(format!("Found {} ROMs", total)))
            .ok();
        tx.send(GuiMessage::Progress(0, total)).ok();

        for (i, rom) in roms.into_iter().enumerate() {
            let result = process_one(&rom, &regions, overwrite, &output_format);
            match result {
                Ok(msg) => {
                    tx.send(GuiMessage::Log(msg)).ok();
                }
                Err(e) => {
                    tx.send(GuiMessage::Log(format!("Error: {}", e))).ok();
                }
            }
            tx.send(GuiMessage::Progress(i + 1, total)).ok();
        }

        tx.send(GuiMessage::Done).ok();
    });
}
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // Collect messages first to avoid borrow issues
        let mut new_messages = vec![];
        if let Some(rx) = &self.rx {
            while let Ok(msg) = rx.try_recv() {
                new_messages.push(msg);
            }
        }

        for msg in new_messages {
            match msg {
                GuiMessage::Log(s) => self.add_log(s),
                GuiMessage::Progress(current, total) => self.progress = (current, total),
                GuiMessage::Done => {
                    self.processing = false;
                    self.add_log("Processing complete.");
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PicoCover GB");
            ui.separator();

            // Drive selection
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(RichText::new("📁 Select Drive").strong());
                ui.add_space(5.0);

                if self.drives.is_empty() {
                    ui.horizontal(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(200, 100, 100),
                            "⚠ No drives with _pico folder found",
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(RichText::new("🔄 Refresh").size(14.0))
                                .clicked()
                            {
                                self.refresh_drives();
                            }
                        });
                    });
                } else {
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_source("drive_selector")
                            .width(ui.available_width() - 100.0)
                            .selected_text(&self.drives[self.selected_drive].path)
                            .show_ui(ui, |ui| {
                                for (idx, drive) in self.drives.iter().enumerate() {
                                    ui.selectable_value(&mut self.selected_drive, idx, &drive.path);
                                }
                            });

                        if ui
                            .button(RichText::new("🔄 Refresh").size(14.0))
                            .clicked()
                        {
                            self.refresh_drives();
                        }
                    });
                }
            });

            ui.add_space(10.0);

            // Options
            ui.checkbox(&mut self.overwrite, "Overwrite existing covers");
            ui.horizontal(|ui| {
                ui.label("Regions:");
                ui.text_edit_singleline(&mut self.regions);
            });
            ui.horizontal(|ui| {
                ui.label("Output format:");
                egui::ComboBox::from_id_source("format")
                    .selected_text(&self.output_format)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.output_format, "bmp".to_string(), "BMP");
                        ui.selectable_value(&mut self.output_format, "png".to_string(), "PNG");
                    });
            });

            ui.add_space(10.0);

            // Start button
            ui.vertical_centered(|ui| {
                let start_enabled = !self.processing && !self.drives.is_empty();
                let button_text = if self.processing {
                    "⏳ Processing..."
                } else {
                    "▶ Start Processing"
                };

                let button = egui::Button::new(RichText::new(button_text).size(16.0))
                    .min_size(egui::vec2(200.0, 40.0));

                if ui.add_enabled(start_enabled, button).clicked() {
                    self.start_processing();
                }
            });

            ui.add_space(10.0);

            // Progress bar
            if self.processing || self.progress.1 > 0 {
                let (current, total) = self.progress;
                ui.add(
                    egui::ProgressBar::new(current as f32 / total as f32)
                        .text(format!("{}/{}", current, total)),
                );
            }

            ui.separator();

            // Logs
            ui.label(RichText::new("Log:").strong());
            ScrollArea::vertical()
                .max_height(300.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for log in &self.logs {
                        ui.label(log);
                    }
                    ui.allocate_space(egui::Vec2::new(0.0, 0.0));
                });

            // Request repaint while processing
            if self.processing {
                ctx.request_repaint();
            }
        });
    }
}

fn process_one(
    rom: &RomInfo,
    regions: &[String],
    overwrite: bool,
    _output_format: &str, // now unused
) -> anyhow::Result<String> {
    use crate::converter::convert_cover;
    use crate::downloader::{cover_exists, download_cover};
    use std::path::PathBuf;

    // Determine drive root
    let drive_root = rom
        .path
        .components()
        .next()
        .map(|c| c.as_os_str())
        .unwrap_or_default();
    let drive_root_path = PathBuf::from(drive_root);

    // Use _pico/covers/gba or _pico/covers/gb
    let covers_dir = match rom.platform {
        crate::scanner::Platform::GBA => drive_root_path.join("_pico").join("covers").join("gba"),
        crate::scanner::Platform::GB | crate::scanner::Platform::GBC => {
            drive_root_path.join("_pico").join("covers").join("gb")
        }
    };

    std::fs::create_dir_all(&covers_dir)?;

    let output_filename = match rom.platform {
        crate::scanner::Platform::GBA => {
            let id = rom.game_id.as_ref().ok_or_else(|| anyhow::anyhow!("No GBA ID"))?;
            format!("{}.bmp", id)
        }
        _ => format!("{}.bmp", rom.file_stem),
    };
    let output_path = covers_dir.join(output_filename);

    if !overwrite && cover_exists(&output_path) {
        return Ok(format!("Skipped {}", rom.path.display()));
    }

    let image_bytes = download_cover(rom, regions, 15, |_| {})?;
    let converted = convert_cover(&image_bytes)?; // <-- single argument
    std::fs::write(&output_path, converted)?;

    Ok(format!("Saved {}", output_path.display()))
}