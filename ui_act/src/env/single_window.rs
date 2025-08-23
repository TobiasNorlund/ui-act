use anyhow::{anyhow, Result};
use image::DynamicImage;
use xcap::Monitor;
use x11rb::protocol::xproto::{ConnectionExt, *};
use x11rb::connection::Connection;

use crate::input::MPXInput;
use crate::env::ComputerEnvironment;
use crate::utils::get_first_monitor;

pub struct SingleWindowEnvironment {
    input: MPXInput,
    monitor: Monitor,
    xwindow_id: u32,
    xconn: x11rb::rust_connection::RustConnection
}

impl SingleWindowEnvironment {
    pub fn create(xwindow_id: u32) -> Result<Self> {
        let (conn, _screen_num) = x11rb::connect(None)?;
        
        // Set window to always be on top
        let env = SingleWindowEnvironment { 
            input: MPXInput::create(&get_first_monitor()?)?, 
            monitor: get_first_monitor()?, 
            xwindow_id: xwindow_id,
            xconn: conn 
        };
        
        env.set_always_on_top(true)?;
        
        Ok(env)
    }
    
    fn set_always_on_top(&self, on_top: bool) -> Result<()> {
        // Set window state to always on top using _NET_WM_STATE_ABOVE
        let atom_above = self.xconn.intern_atom(false, b"_NET_WM_STATE_ABOVE")?.reply()?.atom;
        let atom_wm_state = self.xconn.intern_atom(false, b"_NET_WM_STATE")?.reply()?.atom;
        
        let action = if on_top { 1 } else { 0 }; // 1 = add, 0 = remove
        
        let event = ClientMessageEvent::new(
            32, // format
            self.xwindow_id,
            atom_wm_state,
            [action, atom_above, 0, 1, 0],
        );
        
        // Get the root window
        let setup = self.xconn.setup();
        let root = setup.roots[0].root;
        
        self.xconn.send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
        
        self.xconn.flush()?;

        // Sleep for 500ms to allow the window manager to process the always-on-top request
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        Ok(())
    }
}

impl ComputerEnvironment for SingleWindowEnvironment {
    fn width(&self) -> Result<u32> {
        // Window resolution in framebuffer scale
        let geom = self.xconn.get_geometry(self.xwindow_id)?.reply()?;
        Ok(geom.width as u32)
    }

    fn height(&self) -> Result<u32> {
        let geom = self.xconn.get_geometry(self.xwindow_id)?.reply()?;
        Ok(geom.height as u32)
    }

    fn screenshot(&self) -> Result<image::RgbImage> {
        let geom = self.xconn.get_geometry(self.xwindow_id)?.reply()?;

        let image = self.monitor.capture_image()?;
        // Crop to window geometry
        let image = image::imageops::crop_imm(&image, geom.x as u32, geom.y as u32, geom.width as u32, geom.height as u32);
        // Save the cropped image to a file for debugging
        //image.to_image().save("single_window_screenshot.png")?;
        let image = DynamicImage::ImageRgba8(image.to_image()).to_rgb8();
        //image.save("screenshot.png")?;
        Ok(image)
    }

    fn mouse_move(&mut self, x: u32, y: u32) -> Result<()> {
        // x and y is now relative the window geom
        let geom = self.xconn.get_geometry(self.xwindow_id)?.reply()?;
        if x >= geom.width as u32 || y >= geom.height as u32 {
            return Err(anyhow::anyhow!("Mouse coordinates ({}, {}) exceed window dimensions ({}x{})", x, y, geom.width, geom.height));
        }
        self.input.mouse.mouse_move(geom.x as u32 + x, geom.y as u32 + y)
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

impl Drop for SingleWindowEnvironment {
    fn drop(&mut self) {
        let _ = self.set_always_on_top(false);
    }
}
