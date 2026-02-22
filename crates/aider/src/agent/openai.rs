use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

use crate::event::message_stream_event::MessageStreamEvent;

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    input: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    output: Vec<OpenAIOutput>,
}

#[derive(Debug, Deserialize)]
struct OpenAIOutput {
    content: Vec<OpenAIContent>,
}

#[derive(Debug, Deserialize)]
struct OpenAIContent {
    text: Option<String>,
}

#[derive(Clone)]
pub struct CodeAgent {
    client: Client,
    api_key: String,
}

impl CodeAgent {
    pub fn new_from_env() -> Self {
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not found in environment");

        Self {
            client: Client::new(),
            api_key,
        }
    }
    pub async fn stream_ai_response(
        &self,
        text: &str,
        tx: tokio::sync::mpsc::Sender<MessageStreamEvent>,
    ) -> anyhow::Result<()> {
        let client = reqwest::Client::new();

        let req = OpenAIRequest {
            model: "gpt-4.1-mini".into(),
            input: text.to_string(),
            stream: true,
        };

        let mut stream = client
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await?
            .bytes_stream();

        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer = buffer[pos + 1..].to_string();

                if let Some(mse) = extract_text(&line) {
                    let _ = tx.send(mse).await;
                }
            }
        }

        Ok(())
    }
    pub async fn send(&self, text: &str) -> anyhow::Result<String> {
        let req = OpenAIRequest {
            model: "gpt-4.1-mini".to_string(),
            input: text.to_string(),
            stream: false,
        };

        let res = self
            .client
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await?
            .error_for_status()?;

        let parsed: OpenAIResponse = res.json().await?;

        let output = parsed
            .output
            .get(0)
            .and_then(|out| out.content.get(0))
            .and_then(|content| content.text.as_ref())
            .unwrap_or(&"<no response>".into())
            .clone();

        Ok(output)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum StreamEvent {
    #[serde(rename = "response.created")]
    Created {
        sequence_number: Option<u64>,
        // we ignore other fields for simplicity
    },
    #[serde(rename = "response.completed")]
    Completed { sequence_number: Option<u64> },
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta { delta: String },

    #[serde(rename = "response.refusal.delta")]
    RefusalDelta { delta: String },

    #[serde(rename = "response.error")]
    Error { error: serde_json::Value },

    #[serde(other)]
    Other,
}
fn extract_text(sse: &str) -> Option<MessageStreamEvent> {
    let json_str = sse.trim_start_matches("data: ").trim();

    let event: StreamEvent = serde_json::from_str(json_str).ok()?;

    match event {
        StreamEvent::OutputTextDelta { delta } => {
            Some(MessageStreamEvent::NextWord { word: delta })
        }
        StreamEvent::RefusalDelta { delta } => Some(MessageStreamEvent::NextWord { word: delta }),
        StreamEvent::Created { sequence_number: _ } => Some(MessageStreamEvent::Start),
        StreamEvent::Completed { sequence_number: _ } => Some(MessageStreamEvent::Completed),
        StreamEvent::Error { error } => Some(MessageStreamEvent::Error {
            error: error.to_string(),
        }),
        _ => {
            // println!("{}", json_str);
            None
        }
    }
}
