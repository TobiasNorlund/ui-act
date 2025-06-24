use anyhow::{anyhow, Result};
use std::io::{self, Write};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
enum DeviceError {
    #[error("Failed to create uinput device: {0}")]
    CreationFailed(#[from] uinput::Error),
}

struct MouseDevice {
    device: uinput::Device,
    name: String
}

impl MouseDevice {
    fn create(name: &str, width: i32, height: i32) -> Result<Self> {
        let mut builder = uinput::default()?
            .name(name)?;

        builder = builder.event(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Left))?;
        builder = builder.event(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Right))?;
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

        Ok(MouseDevice {
            device,
            name: name.to_string()
        })
    }

    fn mouse_move(&mut self, x: i32, y: i32) -> Result<()> {
        self.device.send(uinput::event::absolute::Absolute::Position(uinput::event::absolute::Position::X), x)?;
        self.device.send(uinput::event::absolute::Absolute::Position(uinput::event::absolute::Position::Y), y)?;
        self.device.synchronize()?;
        Ok(())
    }

    fn left_click(&mut self) -> Result<()> {
        self.device
            .send(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Left), 1)?;
        self.device.synchronize()?;
        thread::sleep(Duration::from_millis(100));
        self.device
            .send(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Left), 0)?;
        self.device.synchronize()?;
        Ok(())
    }

    fn right_click(&mut self) -> Result<()> {
        self.device
            .send(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Right), 1)?;
        self.device.synchronize()?;
        thread::sleep(Duration::from_millis(100));
        self.device
            .send(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Right), 0)?;
        self.device.synchronize()?;
        Ok(())
    }

    fn double_click(&mut self) -> Result<()> {
        // First click
        self.device
            .send(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Left), 1)?;
        self.device.synchronize()?;
        thread::sleep(Duration::from_millis(50));
        self.device
            .send(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Left), 0)?;
        self.device.synchronize()?;
        
        // Small delay between clicks for double-click recognition
        thread::sleep(Duration::from_millis(50));
        
        // Second click
        self.device
            .send(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Left), 1)?;
        self.device.synchronize()?;
        thread::sleep(Duration::from_millis(50));
        self.device
            .send(uinput::event::controller::Controller::Mouse(uinput::event::controller::Mouse::Left), 0)?;
        self.device.synchronize()?;
        
        Ok(())
    }
}

struct KeyboardDevice {
    device: uinput::Device,
    name: String,
}

impl KeyboardDevice {
    fn create(name: &str) -> Result<Self> {
        let mut builder = uinput::default()?
            .name(name)?;

        for key in [
            uinput::event::keyboard::Key::A, uinput::event::keyboard::Key::B, uinput::event::keyboard::Key::C, uinput::event::keyboard::Key::D, uinput::event::keyboard::Key::E, uinput::event::keyboard::Key::F, uinput::event::keyboard::Key::G, uinput::event::keyboard::Key::H, uinput::event::keyboard::Key::I, uinput::event::keyboard::Key::J,
            uinput::event::keyboard::Key::K, uinput::event::keyboard::Key::L, uinput::event::keyboard::Key::M, uinput::event::keyboard::Key::N, uinput::event::keyboard::Key::O, uinput::event::keyboard::Key::P, uinput::event::keyboard::Key::Q, uinput::event::keyboard::Key::R, uinput::event::keyboard::Key::S, uinput::event::keyboard::Key::T,
            uinput::event::keyboard::Key::U, uinput::event::keyboard::Key::V, uinput::event::keyboard::Key::W, uinput::event::keyboard::Key::X, uinput::event::keyboard::Key::Y, uinput::event::keyboard::Key::Z,
            uinput::event::keyboard::Key::_1, uinput::event::keyboard::Key::_2, uinput::event::keyboard::Key::_3, uinput::event::keyboard::Key::_4, uinput::event::keyboard::Key::_5, uinput::event::keyboard::Key::_6, uinput::event::keyboard::Key::_7, uinput::event::keyboard::Key::_8, uinput::event::keyboard::Key::_9, uinput::event::keyboard::Key::_0,
            uinput::event::keyboard::Key::Space, uinput::event::keyboard::Key::Dot, uinput::event::keyboard::Key::Comma,
            uinput::event::keyboard::Key::LeftControl, uinput::event::keyboard::Key::RightControl,
            uinput::event::keyboard::Key::LeftAlt, uinput::event::keyboard::Key::RightAlt,
            uinput::event::keyboard::Key::LeftShift, uinput::event::keyboard::Key::RightShift,
            uinput::event::keyboard::Key::LeftMeta, uinput::event::keyboard::Key::RightMeta,
            uinput::event::keyboard::Key::Tab, uinput::event::keyboard::Key::Enter, uinput::event::keyboard::Key::Esc,
            uinput::event::keyboard::Key::BackSpace, uinput::event::keyboard::Key::Delete,
            uinput::event::keyboard::Key::Home, uinput::event::keyboard::Key::End,
            uinput::event::keyboard::Key::PageUp, uinput::event::keyboard::Key::PageDown,
            uinput::event::keyboard::Key::Insert,
            uinput::event::keyboard::Key::F1, uinput::event::keyboard::Key::F2, uinput::event::keyboard::Key::F3,
            uinput::event::keyboard::Key::F4, uinput::event::keyboard::Key::F5, uinput::event::keyboard::Key::F6,
            uinput::event::keyboard::Key::F7, uinput::event::keyboard::Key::F8, uinput::event::keyboard::Key::F9,
            uinput::event::keyboard::Key::F10, uinput::event::keyboard::Key::F11, uinput::event::keyboard::Key::F12,
        ] {
            builder = builder.event(key)?;
        }

        let device = builder.create()?;

        // It can take a moment for the device to be ready
        thread::sleep(Duration::from_secs(1));

        Ok(KeyboardDevice {
            device,
            name: name.to_string(),
        })
    }

    fn type_text(&mut self, text: &str) -> Result<()> {
        for c in text.chars() {
            if let Some(key) = char_to_key(c) {
                self.device.send(key, 1)?;
                self.device.synchronize()?;
                thread::sleep(Duration::from_millis(50));
                self.device.send(key, 0)?;
                self.device.synchronize()?;
            }
        }
        Ok(())
    }

    fn press_key(&mut self, key_combination: &str) -> Result<()> {
        let keys = parse_key_combination(key_combination)?;
        
        // Press all keys in sequence
        for key in &keys.keys {
            self.device.send(*key, 1)?;
            self.device.synchronize()?;
        }
        thread::sleep(Duration::from_millis(50));
        
        // Release all keys in reverse sequence
        for key in keys.keys.iter().rev() {
            self.device.send(*key, 0)?;
            self.device.synchronize()?;
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
            "c" => Key::C,
            "v" => Key::V,
            "x" => Key::X,
            "z" => Key::Z,
            "a" => Key::A,
            "f" => Key::F,
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

struct XInputMaster {
    name: String,
    pointer_id: i32,
    keyboard_id: i32
}

impl XInputMaster {
    fn create(name: &str) -> Result<Self> {
        // Try to get the ID if it already exists
        let pointer_id = get_device_id_by_name(&format!("{} pointer", name));
        let keyboard_id = get_device_id_by_name(&format!("{} keyboard", name));

        if let (Ok(pointer_id), Ok(keyboard_id)) = (pointer_id, keyboard_id) {
            println!("master {name} with ids {pointer_id} and {keyboard_id} already exists!");
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

impl Drop for XInputMaster {
    fn drop(&mut self) {
        if let Err(e) = run_xinput(&["remove-master", &self.pointer_id.to_string()]) {
            eprintln!("Failed to remove master device {} (id={}): {}", self.name, self.pointer_id, e);
        } else {
            println!("Removed master device: {} (id={})", self.name, self.pointer_id);
        }
    }
}

fn run_xinput(args: &[&str]) -> Result<()> {
    println!("Running: xinput {}", args.join(" "));
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

fn setup_devices() -> Result<(XInputMaster, MouseDevice, KeyboardDevice)> {
    let mouse = MouseDevice::create("ui-act-mouse", 1920, 1080)?;
    let keyboard = KeyboardDevice::create("ui-act-keyboard")?;
    println!("Created virtual mouse and keyboard");

    let master_device = XInputMaster::create("UI Act")?;
    println!("Created master device pair: {} (pointer id={} keyboard id={})", master_device.name, master_device.pointer_id, master_device.keyboard_id);

    let mouse_id = get_device_id_by_name(&mouse.name)?;
    run_xinput(&["reattach", &mouse_id.to_string(), &master_device.pointer_id.to_string()])?;
    println!("Attached {} (id={}) to {} (id={})", mouse.name, mouse_id, master_device.name, master_device.pointer_id);

    let keyboard_id = get_device_id_by_name(&keyboard.name)?;
    run_xinput(&["reattach", &keyboard_id.to_string(), &master_device.keyboard_id.to_string()])?;
    println!("Attached {} (id={}) to {} (id={})", keyboard.name, keyboard_id, master_device.name, master_device.keyboard_id);

    Ok((master_device, mouse, keyboard))
}

fn main() -> Result<()> {
    let (_master_device, mut mouse, mut keyboard) = setup_devices()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    println!("Starting command interpreter. Type 'exit' or press Ctrl-C to quit.");

    while running.load(Ordering::SeqCst) {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            // EOF (e.g. pipe closed)
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];

        match command {
            "mouse_move" => {
                if parts.len() == 3 {
                    if let (Ok(x), Ok(y)) = (parts[1].parse(), parts[2].parse()) {
                        mouse.mouse_move(x, y)?;
                    } else {
                        eprintln!("Invalid arguments for mouse_move. Expected x y coordinates.");
                    }
                } else {
                    eprintln!("Usage: mouse_move <x> <y>");
                }
            }
            "left_click" => {
                mouse.left_click()?;
            }
            "right_click" => {
                mouse.right_click()?;
            }
            "double_click" => {
                mouse.double_click()?;
            }
            "type" => {
                if parts.len() > 1 {
                    let text = parts[1..].join(" ");
                    keyboard.type_text(&text)?;
                } else {
                    eprintln!("Usage: type <text>");
                }
            }
            "key" => {
                if parts.len() == 2 {
                    keyboard.press_key(parts[1])?;
                } else {
                    eprintln!("Usage: key <key_combination>");
                    eprintln!("Examples: key ctrl+c, key alt+tab, key ctrl+alt+delete");
                }
            }
            "exit" => {
                break;
            }
            _ => {
                eprintln!("Unknown command: {}", command);
            }
        }
    }

    Ok(())
}

fn char_to_key(c: char) -> Option<uinput::event::keyboard::Key> {
    use uinput::event::keyboard::Key;
    match c {
        'a' => Some(Key::A),
        'b' => Some(Key::B),
        'c' => Some(Key::C),
        'd' => Some(Key::D),
        'e' => Some(Key::E),
        'f' => Some(Key::F),
        'g' => Some(Key::G),
        'h' => Some(Key::H),
        'i' => Some(Key::I),
        'j' => Some(Key::J),
        'k' => Some(Key::K),
        'l' => Some(Key::L),
        'm' => Some(Key::M),
        'n' => Some(Key::N),
        'o' => Some(Key::O),
        'p' => Some(Key::P),
        'q' => Some(Key::Q),
        'r' => Some(Key::R),
        's' => Some(Key::S),
        't' => Some(Key::T),
        'u' => Some(Key::U),
        'v' => Some(Key::V),
        'w' => Some(Key::W),
        'x' => Some(Key::X),
        'y' => Some(Key::Y),
        'z' => Some(Key::Z),
        '1' => Some(Key::_1),
        '2' => Some(Key::_2),
        '3' => Some(Key::_3),
        '4' => Some(Key::_4),
        '5' => Some(Key::_5),
        '6' => Some(Key::_6),
        '7' => Some(Key::_7),
        '8' => Some(Key::_8),
        '9' => Some(Key::_9),
        '0' => Some(Key::_0),
        ' ' => Some(Key::Space),
        '.' => Some(Key::Dot),
        ',' => Some(Key::Comma),
        _ => None,
    }
}
