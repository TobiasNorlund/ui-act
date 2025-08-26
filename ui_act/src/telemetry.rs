use std::process::Command;
use serde_json::json;
use reqwest;
use once_cell::sync::Lazy;

const TELEMETRY_ENDPOINT: &str = "https://ui-act-telemetry-1092527829257.europe-north2.run.app/events";

static OS_NAME: Lazy<String> = Lazy::new(|| {
    // Try to get distribution name from /etc/os-release first
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content.lines()
                .find(|line| line.starts_with("PRETTY_NAME="))
                .map(|line| line.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
        })
        .unwrap_or_else(|| {
            "Unknown".to_string()
        })
});

static OS_VERSION: Lazy<String> = Lazy::new(|| {
    // Get VERSION field from /etc/os-release
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content.lines()
                .find(|line| line.starts_with("VERSION="))
                .map(|line| line.trim_start_matches("VERSION=").trim_matches('"').to_string())
        })
        .unwrap_or_else(|| {
            "Unknown".to_string()
        })
});

static GNOME_VERSION: Lazy<String> = Lazy::new(|| {
    Command::new("gnome-shell")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
});


pub async fn post_telemetry(
    session_id: &str, 
    environment: &str,
    event_type: &str, 
    reason: Option<&str>, 
    action_count: Option<u32>
) -> () {
    let mut payload = json!({
        "type": event_type,
        "environment": environment,
        "os_name": *OS_NAME,
        "os_version": *OS_VERSION,
        "gnome_version": *GNOME_VERSION
    });
    
    if let Some(reason_val) = reason {
        payload["reason"] = json!(reason_val);
    }
    
    if let Some(count) = action_count {
        payload["action_count"] = json!(count);
    }

    let telemetry_data = json!({
        "session_id": session_id,
        "client_version": env!("CARGO_PKG_VERSION"),
        "payload": payload
    });

    let telemetry_url = std::env::var("UI_ACT_TELEMETRY_ENDPOINT")
        .unwrap_or_else(|_| TELEMETRY_ENDPOINT.to_string());

    let client = reqwest::Client::new();
    tokio::spawn(async move {
        let _ = client.post(&telemetry_url)
            .header("content-type", "application/json")
            .json(&telemetry_data)
            .send()
            .await;
    });
}
