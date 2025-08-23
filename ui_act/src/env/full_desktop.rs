use anyhow::{anyhow, Result};
use image::DynamicImage;
use xcap::Monitor;
use crate::env::ComputerEnvironment;
use crate::utils::get_first_monitor;
use crate::input::MPXInput;

pub struct FullDesktopEnvironment {
    input: MPXInput,
    monitor: Monitor
}

impl FullDesktopEnvironment {
    pub fn create() -> Result<Self> {
        // TODO: Probably needs to be the upper left most monitor as we don't handle monitor offsets
        let monitor = get_first_monitor()?;
        Ok(FullDesktopEnvironment { input: MPXInput::create(&monitor)?, monitor: monitor })
    }
}

impl ComputerEnvironment for FullDesktopEnvironment {
    fn name(&self) -> String {
        "desktop".to_string()
    }

    fn width(&self) -> Result<u32> {
        Ok(self.monitor.width()?)
    }

    fn height(&self) -> Result<u32> {
        Ok(self.monitor.height()?)
    }

    fn screenshot(&self) -> Result<image::RgbImage> {
        let monitor = get_first_monitor()?;
        let rgba_image = monitor.capture_image()
            .map_err(|e| anyhow!("Failed to capture monitor: {}", e))?;
        let rgb_image = DynamicImage::ImageRgba8(rgba_image).to_rgb8();
        Ok(rgb_image)
    }

    fn mouse_move(&mut self, x: u32, y: u32) -> Result<()> {
        self.input.mouse.mouse_move(x, y)
    }

    fn left_click(&mut self) -> Result<()> {
        self.input.mouse.left_click()
    }

    fn right_click(&mut self) -> Result<()> {
        self.input.mouse.right_click()
    }

    fn double_click(&mut self) -> Result<()> {
        self.input.mouse.double_click()
    }

    fn type_text(&mut self, text: &str) -> Result<()> {
        self.input.keyboard.type_text(text)
    }

    fn press_key(&mut self, key_combination: &str) -> Result<()> {
        self.input.keyboard.press_key(key_combination)
    }
}