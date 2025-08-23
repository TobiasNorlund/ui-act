use anyhow::Result;
use serde_json::json;
use serde::{Serialize, Deserialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use uuid::Uuid;
use crate::telemetry::send_telemetry;
use crate::utils::{img_shrink, rgb_image_to_base64_png};
use crate::env::ComputerEnvironment;


#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
    stop_reason: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
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
    #[serde(rename = "double_click")]
    DoubleClick { coordinate: [u32; 2] },
    #[serde(rename = "type")]
    Type { text: String },
    #[serde(rename = "key")]
    Key { text: String }
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
    pub session_id: String,
    pub action_count: std::cell::Cell<u32>,
}

impl AnthropicAgent {
    pub async fn create() -> Result<Self> {
        let client = reqwest::Client::new();
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY environment variable not set");
        
        let agent = AnthropicAgent {
            client, 
            api_key,
            session_id: Uuid::new_v4().to_string(),
            action_count: std::cell::Cell::new(0),
        };
        
        Ok(agent)
    }

    pub async fn run(&self, env: &mut Box<dyn ComputerEnvironment>, prompt: &str) -> Result<()> {
        let mut screenshot = img_shrink(env.screenshot()?, ANTHROPIC_MAX_WIDTH, ANTHROPIC_MAX_HEIGHT);
        let mut scale: f32 = screenshot.width() as f32 / env.width()? as f32; // Scale relative environment
        let mut messages: Vec<Message> = vec![
            Message { role: "user".to_string(), content: vec![
                ContentBlock::Text { text: prompt.to_string() },
                ContentBlock::Image { source: ImageSource::Base64 {
                    media_type: "image/png".to_string(),
                    data: rgb_image_to_base64_png(&screenshot)?
                }}
            ]}
        ];

        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();

        loop {
            let resp = self.get_response(
                screenshot.width(),
                screenshot.height(),
                &messages
            ).await?;
            if !resp.status().is_success() {
                return Err(anyhow::anyhow!("Anthropic API request failed with status: {}\n{}", resp.status(), resp.text().await?));
            }
            let text = resp.text().await?;
            let res: ApiResponse = match serde_json::from_str(&text) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("Failed to parse response: {e}\nResponse body:\n{text}");
                    return Err(e.into());
                }
            };
            let mut next_message = Message {
                role: "user".to_string(),
                content: vec![]
            };
            for block in &res.content {
                match block {
                    ContentBlock::Text {text} => {
                        println!("\nUI-Act:\n{}", text);
                    }
                    ContentBlock::ToolUse { name, input, id } => {
                        println!("  {:?}", input);
                        if name == "computer" {
                            self.action_count.set(self.action_count.get() + 1);
                            match input {
                                ToolInput::LeftClick { coordinate } => {
                                    let x = (coordinate[0] as f32 / scale).round() as u32;
                                    let y = (coordinate[1] as f32 / scale).round() as u32;
                                    env.mouse_move(x, y)?;
                                    env.left_click()?;
                                }
                                ToolInput::RightClick { coordinate } => {
                                    let x = (coordinate[0] as f32 / scale).round() as u32;
                                    let y = (coordinate[1] as f32 / scale).round() as u32;
                                    env.mouse_move(x, y)?;
                                    env.right_click()?;
                                }
                                ToolInput::DoubleClick { coordinate } => {
                                    let x = (coordinate[0] as f32 / scale).round() as u32;
                                    let y = (coordinate[1] as f32 / scale).round() as u32;
                                    env.mouse_move(x, y)?;
                                    env.double_click()?;
                                }
                                ToolInput::Type { text } => {
                                    env.type_text(&text)?;
                                }
                                ToolInput::Key { text } => {
                                    env.press_key(text)?;
                                }
                                ToolInput::Screenshot => {
                                    // Do nothing, screenshot will be provided below
                                }
                            }

                            // Add a small delay after tool execution to allow UI to update
                            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                            // Send new screenshot as tool result
                            screenshot = img_shrink(env.screenshot()?, ANTHROPIC_MAX_WIDTH, ANTHROPIC_MAX_HEIGHT);
                            scale = screenshot.width() as f32 / env.width()? as f32;
                            next_message.content.push(ContentBlock::ToolResult {
                                content: vec![ContentBlock::Image { source: ImageSource::Base64 {
                                    media_type: "image/png".to_string(),
                                    data: rgb_image_to_base64_png(&screenshot)?
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
                use std::io::{self, Write};
                print!("\n> ");
                io::stdout().flush()?;

                if let Some(line) = lines.next_line().await? {
                    let input = line.trim();
                    if input.is_empty() || input.eq_ignore_ascii_case("exit") {
                        // Send session end telemetry
                        send_telemetry(
                            &self.session_id, 
                            &env.name(), 
                            "session_end", 
                            Some("success"), 
                            Some(self.action_count.get())
                        ).await;
                        break;
                    }
                    next_message.content.push(ContentBlock::Text { text: input.to_string() })
                } else {
                    // EOF (Ctrl-D or terminal closed)
                    send_telemetry(
                        &self.session_id, 
                        &env.name(), 
                        "session_end", 
                        Some("success"), 
                        Some(self.action_count.get())
                    ).await;
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
