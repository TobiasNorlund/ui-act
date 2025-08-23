mod input;
mod device;
mod agent;
mod env;
mod utils;
mod telemetry;

use std::env as std_env;
use crate::agent::AnthropicAgent;
use crate::telemetry::send_telemetry;
use crate::env::full_desktop::FullDesktopEnvironment;
use crate::env::single_window::SingleWindowEnvironment;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    const USAGE: &str = "Usage: ui_act [--window <window_id>] <prompt>";
    
    let mut args = std_env::args();
    let _exe = args.next(); // skip executable name
    
    let mut window_id = None;
    let mut prompt = None;
    
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                eprintln!("{}", USAGE);
                std::process::exit(0);
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
    
    let agent = AnthropicAgent::create().await?;
    
    let mut sighup_stream = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())?;
    tokio::select! {
        result = {
            if let Some(wid) = window_id {
                println!("Running in single window mode with window id: {}", wid);
                let env = Box::new(SingleWindowEnvironment::create(wid)?);
                agent.run(env, &prompt)
            } else {
                println!("Running in full desktop mode");
                let env = Box::new(FullDesktopEnvironment::create()?);
                agent.run(env, &prompt)
            }
        } => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl-C, shutting down gracefully...");
            send_telemetry(&agent.session_id, "session_end", Some("interrupted"), Some(agent.action_count.get())).await?;
        }
        _ = sighup_stream.recv() => {
            println!("Received SIGHUP, shutting down gracefully...");
            send_telemetry(&agent.session_id, "session_end", Some("interrupted"), Some(agent.action_count.get())).await?;
        }
    }

    Ok(())
}