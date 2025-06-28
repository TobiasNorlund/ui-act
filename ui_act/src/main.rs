mod input;
mod device;
mod agent;
mod env;
mod utils;

use std::env as std_env;
use crate::agent::AnthropicAgent;
use crate::env::full_desktop::FullDesktopEnvironment;
use crate::env::single_window::SingleWindowEnvironment;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std_env::args();
    let _exe = args.next(); // skip executable name
    
    let mut window_id = None;
    let mut prompt = None;
    
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--window" => {
                window_id = args.next().map(|id| id.parse::<u32>()).transpose()?;
            }
            _ => {
                prompt = Some(arg);
                break;
            }
        }
    }
    
    let prompt = prompt.unwrap_or_else(|| {
        eprintln!("Usage: ui_act [--window <window_id>] <prompt>");
        std::process::exit(1);
    });
    
    let agent = AnthropicAgent::create().await?;
    
    tokio::select! {
        result = {
            if let Some(wid) = window_id {
                let env = Box::new(SingleWindowEnvironment::create(wid)?);
                agent.run(env, &prompt)
            } else {
                let env = Box::new(FullDesktopEnvironment::create()?);
                agent.run(env, &prompt)
            }
        } => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl-C, shutting down gracefully...");
        }
    }

    Ok(())
}