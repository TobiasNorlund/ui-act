mod input;
mod device;
mod agent;
mod env;
mod utils;
mod telemetry;

use std::env as std_env;
use std::io::{self, Write};
use crate::agent::AnthropicAgent;
use crate::telemetry::post_telemetry;
use crate::env::{ComputerEnvironment, full_desktop::FullDesktopEnvironment, single_window::SingleWindowEnvironment};


const USAGE: &str = "Usage: ui-act [--window <window_id>] [--no-telemetry] [--help] [--version] <prompt>";


fn on_error(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    eprintln!("Press Enter to exit...");
    let _ = io::stdout().flush();
    let mut _input = String::new();
    let _ = io::stdin().read_line(&mut _input);
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn get_signal_handler() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut sighup_stream = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())
        .unwrap_or_else(|e| { on_error(&e.to_string()) });
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("Received Ctrl-C, shutting down gracefully...");
            }
            _ = sighup_stream.recv() => {
                println!("Received SIGHUP, shutting down gracefully...");
            }
        }
    })
}

#[cfg(not(target_os = "linux"))]
fn get_signal_handler() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await;
        println!("Received Ctrl-C, shutting down gracefully...");
    })
}


#[tokio::main]
async fn main() -> () {
    let mut args = std_env::args();
    let _exe = args.next(); // skip executable name
    let mut window_id = None;
    let mut send_telemetry: bool = true;
    
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                eprintln!("{}", USAGE);
                std::process::exit(0);
            }
            "--version" => {
                println!("ui-act version {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--no-telemetry" => {
                send_telemetry = false;
            }
            "--window" => {
                window_id = args.next().map(|id| id.parse::<u32>()).transpose()
                    .unwrap_or_else(|_| { on_error("Unable to parse window as int")});
            }
            _ => {
                // Collect all remaining arguments as the prompt
                let mut prompt_parts = vec![arg];
                prompt_parts.extend(args);
                let prompt = prompt_parts.join(" ");
                return run_with_prompt(prompt, window_id, send_telemetry).await;
            }
        }
    }
    
    on_error("Missing required prompt argument");
}

async fn run_with_prompt(prompt: String, window_id: Option<u32>, send_telemetry: bool) {
    let signal_handle = get_signal_handler();
    
    let agent = AnthropicAgent::create().await
        .unwrap_or_else(|e| { on_error(&e.to_string()) });
    let mut env: Box<dyn ComputerEnvironment> = match window_id {
        Some(wid) => {
            println!("Running in single window mode with window id: {}", wid);
            let env = SingleWindowEnvironment::create(wid)
                .unwrap_or_else(|e| { on_error(&e.to_string()) });
            Box::new(env)
        }
        _ => {
            println!("Running in full desktop mode");
            let env = FullDesktopEnvironment::create()
                .unwrap_or_else(|e| { on_error(&e.to_string()) });
            Box::new(env)
        }
    };
    if send_telemetry {
        post_telemetry(&agent.session_id, &env.name(), "session_start", None, None).await;
    }

    tokio::select! {
        res = agent.run(&mut env, &prompt, send_telemetry) => {
            res.unwrap_or_else(|e| { on_error(&e.to_string()) });
        }
        _ = signal_handle => {
            if send_telemetry {
                post_telemetry(&agent.session_id, &env.name(), "session_end", Some("interrupted"), Some(agent.action_count.get())).await;
            }
        }
    }
}