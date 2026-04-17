<div align="center">

# PicoCover GB

**Automatically download and convert Game Boy / Game Boy Color / Game Boy Advance cover art for your DSPico flashcart**

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.81%2B-orange.svg)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)

*Fetch box art from the Libretro thumbnail repository, convert it to the exact 8‑bit BMP format required by Gameyob and GBArunner3, and place it where your launcher expects it.*

</div>

---

## ✨ Features

- 🖥️ **Simple GUI** – Select your SD card drive from a dropdown and click **Start**.
- 🔍 **Automatic drive detection** – Finds drives that contain a `_pico` folder (the standard Pico flashcart structure).
- 📦 **Covers for GB, GBC, and GBA** – Works with Gameyob (GB/GBC) and GBArunner3 (GBA).
- 🆔 **Smart naming**  
  - **GBA**: Uses the 4‑character game ID from the ROM header (e.g., `BZMP.bmp`) – exactly what GBArunner3 looks for.  
  - **GB/GBC**: Uses the ROM filename (e.g., `Tetris.bmp`) – matches Gameyob’s expectations.
- 🎨 **Proper 8‑bit paletted BMP** – Covers are resized to **106×96**, placed on a **128×96** black canvas, quantized to **256 colors**, and saved in the exact format the DS/Pico hardware can display.
- 📁 **Correct folder structure** – Outputs to `_pico/covers/gba/`, `_pico/covers/gb/`, and `_pico/covers/gbc/` automatically.
- ⚙️ **Optional overwrite control** – Choose whether to replace existing covers.
- 📋 **Live log viewer** – See exactly which URLs are being fetched and the status of each file.

> **Note:** This application is **Windows‑only**. It has been tested on Windows 10/11 with a Pico‑style flashcart running the LNH‑team launcher, Gameyob, and GBArunner3.

---

## 📋 Requirements

- **Windows 10 or 11**
- **Rust 1.81 or later** – [Install Rust](https://rustup.rs/)
- **A Pico‑compatible microSD card** with a `_pico` folder at the root
- **ROM files** (`.gb`, `.gbc`, `.gba`) anywhere on the card
- **Internet connection** (to download covers from Libretro)

---

## 🚀 Installation & Running

### Option 1: Run from source (recommended)

1. **Clone the repository**
   ```bash
   git clone https://github.com/tallcedariptv-png/Pico_GBCovers.git
   cd Pico_GBCovers

2. **Run the application**
   ```bash
   cargo run


### Option 2: Build a standalone executable
 ```bash
cargo build --release



## 🙌 **Credits & Acknowledgements**
**Libretro** – For maintaining the incredible thumbnail repository that makes this tool possible.

**LNH‑team – Creators of Pico Launcher**, the launcher that inspired the original PicoCover.

**Original PicoCover by Scaletta** – The NDS cover downloader that laid the groundwork for this project.

## 📜 **License**
This project is licensed under the **MIT License** – see the LICENSE file for details.
