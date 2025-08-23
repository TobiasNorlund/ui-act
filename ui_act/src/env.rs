use anyhow::Result;

pub mod full_desktop;
pub mod single_window;

pub trait ComputerEnvironment {
    fn name(&self) -> String;
    fn width(&self) -> Result<u32>;
    fn height(&self) -> Result<u32>;
    fn screenshot(&self) -> Result<image::RgbImage>;

    // Mouse actions
    fn mouse_move(&mut self, x: u32, y: u32) -> Result<()>;
    fn left_click(&mut self) -> Result<()>;
    fn right_click(&mut self) -> Result<()>;
    fn double_click(&mut self) -> Result<()>;

    // Keyboard actions
    fn type_text(&mut self, text: &str) -> Result<()>;
    fn press_key(&mut self, key_combination: &str) -> Result<()>;
}