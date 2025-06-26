use anyhow::Result;
use serde_json::json;
use std::io::{self, Write, Cursor};
use base64::{Engine as _, engine::general_purpose};
use serde::{Serialize, Deserialize};
use reqwest::Client;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};

use crate::env::MPXEnvironment;


#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
    stop_reason: String
}

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    role: String,
    content: Vec<ContentBlock>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        source: ImageSource,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: ToolInput,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        content: Vec<ContentBlock>,
        tool_use_id: String,
        is_error: bool
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "action")]
pub enum ToolInput {
    #[serde(rename = "screenshot")]
    Screenshot,
    #[serde(rename = "left_click")]
    LeftClick { coordinate: [u32; 2] },
    #[serde(rename = "right_click")]
    RightClick { coordinate: [u32; 2] },
    #[serde(rename = "type")]
    Type { text: String },
    #[serde(rename = "key")]
    Key { key: String }
    // TODO: Fill out
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ImageSource {
    #[serde(rename = "base64")]
    Base64 {
        media_type: String,
        data: String,
    },
}

const ANTHROPIC_MAX_WIDTH: u32 = 1024;
const ANTHROPIC_MAX_HEIGHT: u32 = 768;

pub struct AnthropicAgent {
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicAgent {
    pub async fn create() -> Result<Self> {
        let client = reqwest::Client::new();
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY environment variable not set");
        Ok(AnthropicAgent {client, api_key})
    }

    pub async fn run(&self, mut env: MPXEnvironment, prompt: &str) -> Result<()> {
        // Note: env.screenshot is likely blocking; adapt to async if possible
        let mut screenshot = env.screenshot(Some(ANTHROPIC_MAX_WIDTH), Some(ANTHROPIC_MAX_HEIGHT))?;
        let mut messages: Vec<Message> = vec![
            Message { role: "user".to_string(), content: vec![
                ContentBlock::Text { text: prompt.to_string() },
                ContentBlock::Image { source: ImageSource::Base64 {
                    media_type: "image/png".to_string(),
                    data: rgba_image_to_base64_png(&screenshot)?
                }}
            ]}
        ];

        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();

        for _ in 0..5 {
            let resp = self.get_response(
                screenshot.width(),
                screenshot.height(),
                &messages
            ).await?;
            if !resp.status().is_success() {
                return Err(anyhow::anyhow!("Anthropic API request failed with status: {}\n{}", resp.status(), resp.text().await?));
            }
            let res: ApiResponse = resp.json().await?;
            let mut next_message = Message {
                role: "user".to_string(),
                content: vec![]
            };
            for block in &res.content {
                match block {
                    ContentBlock::Text {text} => {
                        println!("Assistant: {}", text);
                    }
                    ContentBlock::ToolUse { name, input, id } => {
                        println!("Assistant tool call: {} with input: {:?}", name, input);
                        if name == "computer" {
                            match input {
                                ToolInput::LeftClick { coordinate } => {
                                    let x = coordinate[0] as f32 / screenshot.width() as f32;
                                    let y = coordinate[1] as f32 / screenshot.height() as f32;
                                    env.mouse.mouse_move(x, y)?;
                                    env.mouse.left_click()?;
                                }
                                ToolInput::RightClick { coordinate } => {
                                    let x = coordinate[0] as f32 / screenshot.width() as f32;
                                    let y = coordinate[1] as f32 / screenshot.height() as f32;
                                    env.mouse.mouse_move(x, y)?;
                                    env.mouse.right_click()?;
                                }
                                ToolInput::Type { text } => {
                                    env.keyboard.type_text(&text)?;
                                }
                                ToolInput::Key { key } => {
                                    env.keyboard.press_key(key)?;
                                }
                                ToolInput::Screenshot => {
                                    // Do nothing
                                }
                            }

                            // Send new screenshot as tool result
                            screenshot = env.screenshot(Some(ANTHROPIC_MAX_WIDTH), Some(ANTHROPIC_MAX_HEIGHT))?;
                            next_message.content.push(ContentBlock::ToolResult {
                                content: vec![ContentBlock::Image { source: ImageSource::Base64 {
                                    media_type: "image/png".to_string(),
                                    data: rgba_image_to_base64_png(&screenshot)?
                                }}],
                                tool_use_id: id.clone(),
                                is_error: false
                            })
                        }
                    }
                    ContentBlock::Image { .. } => {
                        println!("Image block in response, ignored");
                    }
                    _ => {
                        return Err(anyhow::anyhow!("Unknown content block variant encountered"));
                    }
                }
            }

            // Add response to messages
            messages.push(Message {
                role: "assistant".to_string(),
                content: res.content
            });

            // Maybe prompt user
            if next_message.content.len() == 0 {
                // No tool result, ask for user input
                print!("> ");
                tokio::io::stdout().flush().await?;

                if let Some(line) = lines.next_line().await? {
                    let input = line.trim();
                    if input.is_empty() {
                        break;
                    }
                    next_message.content.push(ContentBlock::Text { text: input.to_string() })
                } else {
                    break;
                }
            }

            // Add next message to messages list
            messages.push(next_message)
        }

        Ok(())
    }

    pub async fn get_response(&self, display_width_px: u32, display_height_px: u32, messages: &Vec<Message>) -> Result<reqwest::Response, reqwest::Error> {
        let content = json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 1024,
            "tools": [{
                "type": "computer_20250124",
                "name": "computer",
                "display_width_px": display_width_px,
                "display_height_px": display_height_px,
                "display_number": 1
            }],
            "messages": messages
        });

        self.client.post("https://api.anthropic.com/v1/messages")
            .header("content-type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "computer-use-2025-01-24")
            .json(&content)
            .send()
            .await
    }
}

fn rgba_image_to_base64_png(img: &image::RgbaImage) -> Result<String> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    
    img.write_to(&mut cursor, image::ImageFormat::Png)?;
    
    let base64_string = general_purpose::STANDARD.encode(&buffer);
    Ok(base64_string)
}