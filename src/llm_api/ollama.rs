use crate::llm_api::{LLMClient, Message, Role};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    host: String,
    port: u16,
    model: String,
    client: Client,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OllamaMessage {
    role: String,
    content: String,
}

impl From<OllamaMessage> for Message {
    fn from(msg: OllamaMessage) -> Self {
        Message {
            role: Role::from(msg.role.as_str()),
            content: msg.content,
        }
    }
}

impl From<&Message> for OllamaMessage {
    fn from(msg: &Message) -> Self {
        OllamaMessage {
            role: msg.role.as_str().to_string(),
            content: msg.content.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    model: String,
    message: OllamaMessage,
}

impl OllamaClient {
    pub fn new(host: String, port: u16, model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            host,
            port,
            model,
            client,
        }
    }

    pub async fn chat(
        &self,
        system_prompt: Option<String>,
        history: Vec<Message>,
        user_message: String,
    ) -> Result<Message, Box<dyn Error>> {
        let url = format!("http://{}:{}/api/chat", self.host, self.port);

        let mut messages = Vec::new();

        if let Some(system) = system_prompt {
            messages.push(OllamaMessage {
                role: Role::System.as_str().to_string(),
                content: system,
            });
        }

        messages.extend(history.iter().map(OllamaMessage::from));

        messages.push(OllamaMessage {
            role: Role::User.as_str().to_string(),
            content: user_message,
        });

        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
        };

        println!("*** Request:\n{:?}", request);
        println!("*** Request Ende ***");

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<ChatResponse>()
            .await?;

        Ok(response.message.into())
    }
}

#[async_trait]
impl LLMClient for OllamaClient {
    async fn chat(
        &self,
        system_prompt: Option<String>,
        history: Vec<Message>,
        user_message: String,
    ) -> Result<Message, Box<dyn Error>> {
        self.chat(system_prompt, history, user_message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chat_simple_prompt() {
        let client = OllamaClient::new("localhost".to_string(), 11434, "codellama".to_string());

        let response = client
            .chat(None, vec![], "What is Rust?".to_string())
            .await
            .unwrap();

        assert!(!response.content.is_empty());
    }

    #[tokio::test]
    async fn test_chat_with_system_prompt() {
        let client = OllamaClient::new("localhost".to_string(), 11434, "codellama".to_string());

        let response = client
            .chat(
                Some("You are a helpful assistant.".to_string()),
                vec![],
                "What is Rust?".to_string(),
            )
            .await
            .unwrap();

        assert!(!response.content.is_empty());
    }

    #[tokio::test]
    async fn test_conversation() {
        let client = OllamaClient::new("localhost".to_string(), 11434, "codellama".to_string());

        // First message
        let response1 = client
            .chat(None, vec![], "What is Rust?".to_string())
            .await
            .unwrap();

        // Follow-up message using conversation history
        let response2 = client
            .chat(
                None,
                vec![
                    Message {
                        role: Role::User,
                        content: "What is Rust?".to_string(),
                    },
                    response1.clone(),
                ],
                "What are its main features?".to_string(),
            )
            .await
            .unwrap();

        assert!(!response1.content.is_empty());
        assert!(!response2.content.is_empty());
    }
}
