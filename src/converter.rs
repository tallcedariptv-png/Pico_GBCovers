use anyhow::Result;
use color_quant::NeuQuant;
use image::{ImageBuffer, Rgba, RgbaImage};

pub fn convert_cover(image_bytes: &[u8]) -> Result<Vec<u8>> {
    // Load image
    let img = image::load_from_memory(image_bytes)?;

    // Step 1: Force-resize to 106x96 (ignore aspect ratio)
    let resized = img.resize_exact(106, 96, image::imageops::FilterType::Lanczos3);

    // Step 2: Create 128x96 black canvas
    let mut canvas: RgbaImage = ImageBuffer::from_pixel(128, 96, Rgba([0, 0, 0, 255]));

    // Step 3: Place resized image at top-left (northwest gravity)
    image::imageops::overlay(&mut canvas, &resized, 0, 0);

    // Step 4: Flatten RGBA pixels
    let rgba_pixels: Vec<u8> = canvas.pixels().flat_map(|p| p.0).collect();

    // Step 5: Quantize to 256 colors using NeuQuant
    let nq = NeuQuant::new(10, 256, &rgba_pixels);
    let palette = nq.color_map_rgba(); // RGBA palette bytes

    let indices: Vec<u8> = rgba_pixels
        .chunks(4)
        .map(|chunk| nq.index_of(chunk) as u8)
        .collect();

    // Step 6: Write 8-bit paletted BMP
    Ok(write_paletted_bmp(128, 96, &palette, &indices))
}

fn write_paletted_bmp(width: u32, height: u32, palette: &[u8], indices: &[u8]) -> Vec<u8> {
    let palette_colors = palette.len() / 4;
    let palette_entries = 256;

    let row_padding = (4 - (width as usize % 4)) % 4;
    let row_size = width as usize + row_padding;
    let image_size = row_size * height as usize;
    let header_size = 14 + 40 + palette_entries * 4;
    let file_size = header_size + image_size;

    let mut data = Vec::with_capacity(file_size);

    // BMP File Header
    data.extend_from_slice(b"BM");
    data.extend_from_slice(&(file_size as u32).to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&(header_size as u32).to_le_bytes());

    // DIB Header
    data.extend_from_slice(&40u32.to_le_bytes());
    data.extend_from_slice(&(width as i32).to_le_bytes());
    data.extend_from_slice(&(height as i32).to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&8u16.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    data.extend_from_slice(&(image_size as u32).to_le_bytes());
    data.extend_from_slice(&2835i32.to_le_bytes()); // horizontal DPI
    data.extend_from_slice(&2835i32.to_le_bytes()); // vertical DPI
    data.extend_from_slice(&(palette_entries as u32).to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());

    // Palette (BGR0)
    for i in 0..palette_entries {
        if i < palette_colors {
            let base = i * 4;
            data.push(palette[base + 2]); // B
            data.push(palette[base + 1]); // G
            data.push(palette[base]);     // R
        } else {
            data.extend_from_slice(&[0u8, 0u8, 0u8]);
        }
        data.push(0u8); // reserved
    }

    // Pixel data (bottom-up)
    let width_usize = width as usize;
    for row in (0..height as usize).rev() {
        let start = row * width_usize;
        let end = start + width_usize;
        data.extend_from_slice(&indices[start..end]);
        if row_padding > 0 {
            data.extend(std::iter::repeat_n(0u8, row_padding));
        }
    }

    data
}
