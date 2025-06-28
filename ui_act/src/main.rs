mod input;
mod device;
mod agent;
mod env;
mod utils;

use std::env as std_env;
use crate::agent::AnthropicAgent;
use crate::env::full_desktop::FullDesktopEnvironment;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std_env::args();
    let _exe = args.next(); // skip executable name
    let prompt = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("Usage: ui_act <prompt>");
            std::process::exit(1);
        }
    };
    let env = FullDesktopEnvironment::create()?;
    let agent = AnthropicAgent::create().await?;
    tokio::select! {
        result = agent.run(env, &prompt) => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl-C, shutting down gracefully...");
        }
    }

    Ok(())
}