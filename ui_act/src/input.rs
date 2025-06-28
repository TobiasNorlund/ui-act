use anyhow::Result;
use xcap::Monitor;
use crate::device::{XInputMaster, MouseDevice, KeyboardDevice, run_xinput};


pub struct MPXInput {
    master: XInputMaster,
    pub mouse: MouseDevice,
    pub keyboard: KeyboardDevice,
}

impl MPXInput {
    pub fn create(monitor: &Monitor) -> Result<Self> {
        // Note: Uses the screen resolution of the first monitor
        // Multiply by scale_factor to get framebuffer size
        let scale = monitor.scale_factor()?;
        let width = (monitor.width()? as f32 * scale) as i32;
        let height = (monitor.height()? as f32 * scale) as i32;
        let mouse = MouseDevice::create("ui-act-mouse", width, height)?;
        let keyboard = KeyboardDevice::create("ui-act-keyboard")?;
        println!("Created virtual mouse and keyboard");
    
        let master = XInputMaster::create("UI Act")?;
        println!("Created master device pair: {} (pointer id={} keyboard id={})", master.name, master.pointer_id, master.keyboard_id);
    
        run_xinput(&["reattach", &mouse.id.to_string(), &master.pointer_id.to_string()])?;
        //println!("Attached {} (id={}) to {} (id={})", mouse.name, mouse.id, master.name, master.pointer_id);
        run_xinput(&["reattach", &keyboard.id.to_string(), &master.keyboard_id.to_string()])?;
        //println!("Attached {} (id={}) to {} (id={})", keyboard.name, keyboard.id, master.name, master.keyboard_id);
    
        Ok(MPXInput { master, mouse, keyboard })
    }
}
