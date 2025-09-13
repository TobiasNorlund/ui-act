use anyhow::Result;
use std::time::Duration;

pub mod full_desktop;
pub mod single_window;

pub trait ComputerEnvironment {
    fn name(&self) -> String;
    fn width(&self) -> Result<u32>;
    fn height(&self) -> Result<u32>;
    
    // General actions
    fn screenshot(&self) -> Result<image::RgbImage>;
    fn wait(&mut self, duration: Duration) -> Result<()>;
    fn scroll(&mut self, direction: &str, amount: u32) -> Result<()>;

    // Mouse actions
    fn mouse_move(&mut self, x: u32, y: u32) -> Result<()>;
    fn cursor_position(&mut self) -> Result<(u32, u32)>;

    fn left_mouse_down(&mut self) -> Result<()>;
    fn left_mouse_up(&mut self) -> Result<()>;
    fn left_click(&mut self) -> Result<()>;
    fn left_click_drag(&mut self, x: u32, y: u32) -> Result<()>;
    
    fn right_click(&mut self) -> Result<()>;
    fn middle_click(&mut self) -> Result<()>;
    fn double_click(&mut self) -> Result<()>;
    fn triple_click(&mut self) -> Result<()>;
    
    // Keyboard actions
    fn hold_key(&mut self, key: &str, duration: Duration) -> Result<()>;
    fn type_text(&mut self, text: &str) -> Result<()>;
    fn press_key(&mut self, key_combination: &str) -> Result<()>;

}