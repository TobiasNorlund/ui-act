use anyhow::{anyhow, Result};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
//use image::ImageEncoder;

mod env;
mod agent;


fn main() -> Result<()> {
    let mut env = env::MPXEnvironment::create()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    agent::run_agent(env, "Close the window. Here is a screenshot of my desktop, use left_click action")?;
    return Ok(());

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
                        env.mouse.mouse_move(x, y)?;
                    } else {
                        eprintln!("Invalid arguments for mouse_move. Expected x y coordinates.");
                    }
                } else {
                    eprintln!("Usage: mouse_move <x> <y>");
                }
            }
            "left_click" => {
                env.mouse.left_click()?;
            }
            "right_click" => {
                env.mouse.right_click()?;
            }
            "double_click" => {
                env.mouse.double_click()?;
            }
            "type" => {
                if parts.len() > 1 {
                    let text = parts[1..].join(" ");
                    env.keyboard.type_text(&text)?;
                } else {
                    eprintln!("Usage: type <text>");
                }
            }
            "key" => {
                if parts.len() == 2 {
                    env.keyboard.press_key(parts[1])?;
                } else {
                    eprintln!("Usage: key <key_combination>");
                    eprintln!("Examples: key ctrl+c, key alt+tab, key ctrl+alt+delete");
                }
            }
            "screenshot" => {
                let _ = env.screenshot(Some(1024), Some(768));
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
