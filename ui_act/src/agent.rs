use serde_json::{json, Value};
use anyhow::Result;
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose};
use serde::Deserialize;

use crate::env::MPXEnvironment;


#[derive(Debug, Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
    // other fields omitted
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: ToolInput,
    },
    // You can add more variants as needed
}

#[derive(Debug, Deserialize)]
struct ToolInput {
    action: String,
    #[serde(default)]
    coordinate: Option<[i32; 2]>,
    // Add more fields as needed
}

pub fn run_agent(mut env: MPXEnvironment, prompt: &str) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable not set");

    let screenshot = env.screenshot(Some(1024), Some(768))?;
    let mut messages = json!([
        {"role": "user", "content": [
            {"type": "text", "text": prompt},
            {"type": "image", "source": {
                "type": "base64", 
                "media_type": "image/png", 
                "data": rgba_image_to_base64_png(&screenshot)?
            }},
        ]}
    ]);
    //messages.as_array_mut().unwrap().push(json!({"a": 1}));

    let res: ApiResponse = client.post("https://api.anthropic.com/v1/messages")
        .header("content-type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "computer-use-2025-01-24")
        .json(&json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 1024,
            "tools": [{
                "type": "computer_20250124",
                "name": "computer",
                "display_width_px": screenshot.height(),
                "display_height_px": screenshot.width(),
                "display_number": 1
            }],
            "messages": messages
        }))
        .send()?
        .json()?;

    println!("{:?}", res);

    // Handle tool_use if present
    for block in &res.content {
        if let ContentBlock::ToolUse { name, input, .. } = block {
            println!("Tool call: {} with input: {:?}", name, input);
            if name == "computer" && input.action == "left_click" {
                if let Some([x, y]) = input.coordinate {
                    println!("Performing left click at ({}, {})...", x, y);
                    let (x, y) = (x as f32 / screenshot.width() as f32, y as f32 / screenshot.height() as f32);
                    env.mouse.mouse_move(x, y)?;
                } else {
                    println!("Performing left click at current position...");
                }
                //env.mouse.left_click()?;
            }
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(5));

    Ok(())
}


fn rgba_image_to_base64_png(img: &image::RgbaImage) -> Result<String> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    
    img.write_to(&mut cursor, image::ImageFormat::Png)?;
    
    let base64_string = general_purpose::STANDARD.encode(&buffer);
    Ok(base64_string)
}