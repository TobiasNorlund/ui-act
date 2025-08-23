mod input;
mod device;
mod agent;
mod env;
mod utils;
mod telemetry;

use std::env as std_env;
use crate::agent::AnthropicAgent;
use crate::telemetry::post_telemetry;
use crate::env::{ComputerEnvironment, full_desktop::FullDesktopEnvironment, single_window::SingleWindowEnvironment};


const USAGE: &str = "Usage: ui_act [--window <window_id>] [--no-telemetry] [--help] <prompt>";


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    
    let mut args = std_env::args();
    let _exe = args.next(); // skip executable name
    
    let mut window_id = None;
    let mut prompt = None;
    let mut send_telemetry: bool = true;
    
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                eprintln!("{}", USAGE);
                std::process::exit(0);
            }
            "--no-telemetry" => {
                send_telemetry = false;
            }
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
        eprintln!("Error: Missing required prompt argument");
        eprintln!("{}", USAGE);
        std::process::exit(1);
    });
        
    let mut sighup_stream = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())?;
    let agent = AnthropicAgent::create().await?;
    let mut env: Box<dyn ComputerEnvironment> = match window_id {
        Some(wid) => {
            println!("Running in single window mode with window id: {}", wid);
            Box::new(SingleWindowEnvironment::create(wid)?)
        }
        _ => {
            println!("Running in full desktop mode");
            Box::new(FullDesktopEnvironment::create()?)
        }
    };
    if send_telemetry {
        post_telemetry(&agent.session_id, &env.name(), "session_start", None, None).await;
    }
    
    tokio::select! {
        _ = agent.run(&mut env, &prompt, send_telemetry) => {}
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl-C, shutting down gracefully...");
            if send_telemetry {
                post_telemetry(&agent.session_id, &env.name(), "session_end", Some("interrupted"), Some(agent.action_count.get())).await;
            }
        }
        _ = sighup_stream.recv() => {
            println!("Received SIGHUP, shutting down gracefully...");
            if send_telemetry {
                post_telemetry(&agent.session_id, &env.name(), "session_end", Some("interrupted"), Some(agent.action_count.get())).await;
            }
        }
    }

    Ok(())
}