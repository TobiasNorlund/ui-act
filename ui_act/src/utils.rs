use std::io::Cursor;
use anyhow::{anyhow, Result};
use base64::engine::general_purpose;
use base64::Engine as _;
use image::RgbImage;
use image::imageops::resize;
use image::imageops::FilterType;


pub fn get_first_monitor() -> Result<xcap::Monitor> {
    xcap::Monitor::all()
        .map_err(|e| anyhow!("Failed to enumerate monitors: {}", e))?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No monitors found"))
}

pub fn img_shrink(img: RgbImage, max_width: u32, max_height: u32) -> RgbImage {
    let (width, height) = (img.width(), img.height());
    let scale_w = max_width as f32 / width as f32;
    let scale_h = max_height as f32 / height as f32;
    let scale = scale_w.min(scale_h).min(1.0); // Don't upscale

    let new_width = (width as f32 * scale).round() as u32;
    let new_height = (height as f32 * scale).round() as u32;

    if new_width != width || new_height != height {
        resize(&img, new_width, new_height, FilterType::Triangle)
    } else {
        img
    }
}

pub fn rgb_image_to_base64_png(img: &image::RgbImage) -> Result<String> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    
    img.write_to(&mut cursor, image::ImageFormat::Png)?;
    
    let base64_string = general_purpose::STANDARD.encode(&buffer);
    Ok(base64_string)
}