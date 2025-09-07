use anyhow::{anyhow, Result};
use std::thread;
use std::time::Duration;
use thiserror::Error;
use std::process::Command;


#[derive(Error, Debug)]
enum DeviceError {
    #[error("Failed to create uinput device: {0}")]
    CreationFailed(#[from] uinput::Error),
}

pub struct XInputMaster {
    pub name: String,
    pub pointer_id: i32,
    pub keyboard_id: i32
}

impl XInputMaster {
    pub fn create(name: &str) -> Result<Self> {
        // Try to get the ID if it already exists
        let pointer_id = get_device_id_by_name(&format!("{} pointer", name));
        let keyboard_id = get_device_id_by_name(&format!("{} keyboard", name));

        if let (Ok(pointer_id), Ok(keyboard_id)) = (pointer_id, keyboard_id) {
            //println!("master {name} with ids {pointer_id} and {keyboard_id} already exists!");
            return Ok(XInputMaster { name: name.to_string(), pointer_id, keyboard_id });
        }
        // Otherwise, create it
        run_xinput(&["create-master", name])?;
        // Wait a moment for the device to appear
        thread::sleep(Duration::from_millis(200));
        let pointer_id = get_device_id_by_name(&format!("{} pointer", name))?;
        let keyboard_id = get_device_id_by_name(&format!("{} keyboard", name))?;
        
        Ok(XInputMaster { name: name.to_string(), pointer_id, keyboard_id })
    }
}

fn get_device_id_by_name(name: &str) -> Result<i32> {
    let output = Command::new("xinput").arg("list").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains(name) {
            if let Some(id_part) = line.split_whitespace().find(|s| s.starts_with("id=")) {
                if let Some(id_str) = id_part.strip_prefix("id=") {
                    if let Ok(id) = id_str.parse() {
                        return Ok(id);
                    }
                }
            }
        }
    }
    Err(anyhow!("Device '{}' not found in xinput list", name))
}

pub fn run_xinput(args: &[&str]) -> Result<()> {
    //println!("Running: xinput {}", args.join(" "));
    let output = Command::new("xinput").args(args).output()?;
    if !output.status.success() {
        return Err(anyhow!(
            "xinput command failed with status: {}\\nStdout: {}\\nStderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}


// General Device Statics
pub static SYNC_DELAY: Duration = Duration::from_micros(1); // should relinquish thread control to read the last sync report


// Mouse Device Statics

static CLICK_DELAY: Duration = Duration::from_millis(100);
static MULTI_CLICK_DELAY: Duration = Duration::from_millis(50);

pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub enum ScrollDirection {
    Up,
    Down,
    Right,
    Left,
}

impl ScrollDirection {
    pub fn from_str(direction: &str) -> Result<Self> {
        match direction {
            "up" => Ok(ScrollDirection::Up),
            "down" => Ok(ScrollDirection::Down),
            "right" => Ok(ScrollDirection::Right),
            "left" => Ok(ScrollDirection::Left),
            _ => Err(anyhow!("Invalid scroll direction: {}", direction)),
        }
    }

    pub fn multiplier(&self) -> i32 {
        match self {
            ScrollDirection::Up => 1,
            ScrollDirection::Down => -1,
            ScrollDirection::Right => 1,
            ScrollDirection::Left => -1,
        }
    }
}

pub struct MouseDevice {
    pub id: i32,
    pub name: String,
    device: uinput::Device
}



impl MouseDevice {
    pub fn create(name: &str, width: i32, height: i32) -> Result<Self> {
        let mut builder = uinput::default()?
            .name(name)?;

        builder = builder.event(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Left))?;
        builder = builder.event(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Right))?;
        builder = builder.event(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Middle))?;
        builder = builder.event(uinput::event::relative::Relative::Wheel(uinput::event::relative::Wheel::Horizontal))?;
        builder = builder.event(uinput::event::relative::Relative::Wheel(uinput::event::relative::Wheel::Vertical))?;
        builder = builder.event(uinput::event::absolute::Absolute::Position(uinput::event::absolute::Position::X))?
            .min(0)
            .max(width)
            .fuzz(0)
            .flat(0);
        builder = builder.event(uinput::event::absolute::Absolute::Position(uinput::event::absolute::Position::Y))?
            .min(0)
            .max(height)
            .fuzz(0)
            .flat(0);

        let device = builder.create()?;

        // It can take a moment for the device to be ready
        thread::sleep(Duration::from_secs(1));

        let id = get_device_id_by_name(name)?;

        Ok(MouseDevice {
            id,
            name: name.to_string(),
            device
        })
    }

    pub fn mouse_move(&mut self, x: u32, y: u32) -> Result<()> {
        self.device.send(uinput::event::absolute::Absolute::Position(uinput::event::absolute::Position::X), x as i32)?;
        self.device.send(uinput::event::absolute::Absolute::Position(uinput::event::absolute::Position::Y), y as i32)?;
        self.device.synchronize()?;
        Ok(())
    }

    pub fn scroll(&mut self, scroll_direction: ScrollDirection, amount: u32) -> Result<()> {
        /* 
        uinput lib incorrectly uses legacy wheel values 0x06 (REL_HWHEEL) and 0x08 (REL_WHEEL)
        and should use the Hi-Res values 0x0b (REL_HWHEEL_HI_RES) and 0x0c (REL_WHEEL_HI_RES) instead
        when possible. Something for the future to fix?

        Documentation for kernel 6.8 (shipped with Ubuntu 24.04)
        ref: https://github.com/torvalds/linux/blob/v6.8/include/uapi/linux/input-event-codes.h#L833
        */
        let wheel = match scroll_direction {
            ScrollDirection::Up => uinput::event::relative::Wheel::Vertical,
            ScrollDirection::Down => uinput::event::relative::Wheel::Vertical,
            ScrollDirection::Right => uinput::event::relative::Wheel::Horizontal,
            ScrollDirection::Left => uinput::event::relative::Wheel::Horizontal,
        };
        self.device.send(uinput::event::relative::Relative::Wheel(wheel), amount as i32 * scroll_direction.multiplier())?;
        self.device.synchronize()?;
        Ok(())
    }

    fn mouse_button_down(&mut self, button: uinput::event::controller::Mouse) -> Result<()> {
        let send_result = self.device.send(uinput::event::controller::Controller::Mouse(button), 1);
        self.device.synchronize()?;
        thread::sleep(SYNC_DELAY);
        Ok(())
    }

    fn mouse_button_up(&mut self, button: uinput::event::controller::Mouse) -> Result<()> {
        let send_result = self.device.send(uinput::event::controller::Controller::Mouse(button), 0);
        self.device.synchronize()?;
        thread::sleep(SYNC_DELAY);
        Ok(())
    }
    


    pub fn mouse_down(&mut self, button: MouseButton) -> Result<()> {
        let button = match button {
            MouseButton::Left => uinput::event::controller::Mouse::Left,
            MouseButton::Right => uinput::event::controller::Mouse::Right,
            MouseButton::Middle => uinput::event::controller::Mouse::Middle,
        };
        self.mouse_button_down(button)?;
        Ok(())
    }
    
    pub fn mouse_up(&mut self, button: MouseButton) -> Result<()> {
        let button = match button {
            MouseButton::Left => uinput::event::controller::Mouse::Left,
            MouseButton::Right => uinput::event::controller::Mouse::Right,
            MouseButton::Middle => uinput::event::controller::Mouse::Middle,
        };
        self.mouse_button_up(button)?;
        Ok(())
    }
    
    pub fn click(&mut self, button: MouseButton) -> Result<()> {
        let button = &match button {
            MouseButton::Left => uinput::event::controller::Mouse::Left,
            MouseButton::Right => uinput::event::controller::Mouse::Right,
            MouseButton::Middle => uinput::event::controller::Mouse::Middle,
        };
        self.mouse_button_down(*button)?;
        thread::sleep(CLICK_DELAY);
        self.mouse_button_up(*button)?;
        Ok(())
    }

    pub fn click_drag(&mut self, button: MouseButton, x: u32, y: u32) -> Result<()> {
        let button = &match button {
            MouseButton::Left => uinput::event::controller::Mouse::Left,
            MouseButton::Right => uinput::event::controller::Mouse::Right,
            MouseButton::Middle => uinput::event::controller::Mouse::Middle,
        };
        self.mouse_button_down(*button)?;
        thread::sleep(CLICK_DELAY);
        self.mouse_move(x, y)?;
        thread::sleep(CLICK_DELAY);
        self.mouse_button_up(*button)?;
        Ok(())
    }

    pub fn double_click(&mut self) -> Result<()> {
        self.click(MouseButton::Left)?;
        thread::sleep(MULTI_CLICK_DELAY);
        self.click(MouseButton::Left)?;
        Ok(())
    }

    pub fn triple_click(&mut self) -> Result<()> {
        self.click(MouseButton::Left)?;
        thread::sleep(MULTI_CLICK_DELAY);
        self.click(MouseButton::Left)?;
        thread::sleep(MULTI_CLICK_DELAY);
        self.click(MouseButton::Left)?;
        Ok(())
    }
}


static KEY_PRESS_DELAY: Duration = Duration::from_millis(50);

pub struct KeyboardDevice {
    pub id: i32,
    pub name: String,
    device: uinput::Device,
}

impl KeyboardDevice {
    pub fn create(name: &str) -> Result<Self> {
        let mut builder = uinput::default()?
            .name(name)?;

        for key in uinput::event::keyboard::Key::iter_variants() {
            builder = builder.event(key)?;
        }

        let device = builder.create()?;

        // It can take a moment for the device to be ready
        thread::sleep(Duration::from_secs(1));

        let id = get_device_id_by_name(name)?;

        Ok(KeyboardDevice {
            id,
            name: name.to_string(),
            device,
        })
    }

    pub fn type_text(&mut self, text: &str) -> Result<()> {
       for c in text.chars() {
            let keys = char_to_keys(c);
            if keys.is_empty() {
                continue; // skip unsupported chars
            }
            // Press all modifier keys except the last (main) key
            for key in &keys[..keys.len().saturating_sub(1)] {
                self.key_down(*key)?;
            }
            // Press the main key
            let main_key = keys.last().unwrap();
            self.key_down(*main_key)?;

            thread::sleep(KEY_PRESS_DELAY);
            
            // Release the main key
            self.key_up(*main_key)?;
            // Release modifiers in reverse order
            for key in keys[..keys.len().saturating_sub(1)].iter().rev() {
                self.key_up(*key)?;
            }
            thread::sleep(KEY_PRESS_DELAY);
        }
        Ok(())
    }

    fn key_down(&mut self, key: uinput::event::keyboard::Key) -> Result<()> {
        self.device.send(key, 1)?;
        self.device.synchronize()?;
        Ok(())
    }
    
    fn key_up(&mut self, key: uinput::event::keyboard::Key) -> Result<()> {
        self.device.send(key, 0)?;
        self.device.synchronize()?;
        Ok(())
    }

    pub fn hold_key(&mut self, key_combination: &str, duration: Duration) -> Result<()> {
        let keys = parse_key_combination(key_combination)?;
        for key in keys.keys.iter() {
            self.key_down(*key)?;
        }

        thread::sleep(duration);

        for key in keys.keys.iter().rev() {
            self.key_up(*key)?;
        }
        Ok(())
    }

    pub fn press_key(&mut self, key_combination: &str) -> Result<()> {
        let keys = parse_key_combination(key_combination)?;
        
        // Press all keys in sequence
        for key in keys.keys.iter() {
            self.key_down(*key)?;
        }
        thread::sleep(KEY_PRESS_DELAY);
        
        // Release all keys in reverse sequence
        for key in keys.keys.iter().rev() {
            self.key_up(*key)?;
        }
        
        Ok(())
    }
}


struct KeyCombination {
    keys: Vec<uinput::event::keyboard::Key>,
}

fn parse_key_combination(combination: &str) -> Result<KeyCombination> {
    use uinput::event::keyboard::Key;
    
    let parts: Vec<&str> = combination.split('+').collect();
    let mut keys = Vec::new();
    
    for part in parts {
        let key = match part.to_lowercase().as_str() {
            "ctrl" | "control" => Key::LeftControl,
            "alt" => Key::LeftAlt,
            "shift" => Key::LeftShift,
            "meta" | "win" | "super" => Key::LeftMeta,
            "0" => Key::_0,
            "1" => Key::_1,
            "2" => Key::_2,
            "3" => Key::_3,
            "4" => Key::_4,
            "5" => Key::_5,
            "6" => Key::_6,
            "7" => Key::_7,
            "8" => Key::_8,
            "9" => Key::_9,
            "." => Key::Dot,
            "," => Key::Comma,
            " " => Key::Space,
            "a" => Key::A,
            "b" => Key::B,
            "c" => Key::C,
            "d" => Key::D,
            "e" => Key::E,
            "f" => Key::F,
            "g" => Key::G,
            "h" => Key::H,
            "i" => Key::I,
            "j" => Key::J,
            "k" => Key::K,
            "l" => Key::L,
            "m" => Key::M,
            "n" => Key::N,
            "o" => Key::O,
            "p" => Key::P,
            "q" => Key::Q,
            "r" => Key::R,
            "s" => Key::S,
            "t" => Key::T,
            "u" => Key::U,
            "v" => Key::V,
            "w" => Key::W,
            "x" => Key::X,
            "y" => Key::Y,
            "z" => Key::Z,
            "f1" => Key::F1,
            "f2" => Key::F2,
            "f3" => Key::F3,
            "f4" => Key::F4,
            "f5" => Key::F5,
            "f6" => Key::F6,
            "f7" => Key::F7,
            "f8" => Key::F8,
            "f9" => Key::F9,
            "f10" => Key::F10,
            "f11" => Key::F11,
            "f12" => Key::F12,
            "tab" => Key::Tab,
            "enter" | "return" => Key::Enter,
            "escape" | "esc" => Key::Esc,
            "backspace" => Key::BackSpace,
            "delete" | "del" => Key::Delete,
            "home" => Key::Home,
            "end" => Key::End,
            "pageup" => Key::PageUp,
            "pagedown" => Key::PageDown,
            "insert" => Key::Insert,
            _ => return Err(anyhow!("Unknown key: {}", part)),
        };
        keys.push(key);
    }
    
    Ok(KeyCombination { keys })
}

fn char_to_keys(c: char) -> Vec<uinput::event::keyboard::Key> {
    use uinput::event::keyboard::Key;
    let mut keys = vec![];
    if c.is_uppercase() {
        keys.push(Key::LeftShift)
    }
    match c.to_ascii_lowercase() {
        'a' => keys.push(Key::A),
        'b' => keys.push(Key::B),
        'c' => keys.push(Key::C),
        'd' => keys.push(Key::D),
        'e' => keys.push(Key::E),
        'f' => keys.push(Key::F),
        'g' => keys.push(Key::G),
        'h' => keys.push(Key::H),
        'i' => keys.push(Key::I),
        'j' => keys.push(Key::J),
        'k' => keys.push(Key::K),
        'l' => keys.push(Key::L),
        'm' => keys.push(Key::M),
        'n' => keys.push(Key::N),
        'o' => keys.push(Key::O),
        'p' => keys.push(Key::P),
        'q' => keys.push(Key::Q),
        'r' => keys.push(Key::R),
        's' => keys.push(Key::S),
        't' => keys.push(Key::T),
        'u' => keys.push(Key::U),
        'v' => keys.push(Key::V),
        'w' => keys.push(Key::W),
        'x' => keys.push(Key::X),
        'y' => keys.push(Key::Y),
        'z' => keys.push(Key::Z),
        '1' => keys.push(Key::_1),
        '2' => keys.push(Key::_2),
        '3' => keys.push(Key::_3),
        '4' => keys.push(Key::_4),
        '5' => keys.push(Key::_5),
        '6' => keys.push(Key::_6),
        '7' => keys.push(Key::_7),
        '8' => keys.push(Key::_8),
        '9' => keys.push(Key::_9),
        '0' => keys.push(Key::_0),
        ' ' => keys.push(Key::Space),
        '.' => keys.push(Key::Dot),
        ';' => keys.push(Key::SemiColon),
        ':' => {keys.push(Key::LeftShift); keys.push(Key::SemiColon)},
        '-' => keys.push(Key::Minus),
        '_' => {keys.push(Key::LeftShift); keys.push(Key::Minus)},
        ',' => keys.push(Key::Comma),
        '/' => keys.push(Key::Slash),
        '?' => {keys.push(Key::LeftShift); keys.push(Key::Slash)},
        '\n' => keys.push(Key::Enter),
        _ => ()
    };
    keys
}
