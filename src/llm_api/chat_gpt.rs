use crate::llm_api::{LLMClient, Message, Role};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ChatGPTClient {
    api_key: String,
    model: String,
    client: Client,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatGPTMessage {
    role: String,
    content: String,
}

impl From<ChatGPTMessage> for Message {
    fn from(msg: ChatGPTMessage) -> Self {
        Message {
            role: Role::from(msg.role.as_str()),
            content: msg.content,
        }
    }
}

impl From<&Message> for ChatGPTMessage {
    fn from(msg: &Message) -> Self {
        ChatGPTMessage {
            role: msg.role.as_str().to_string(),
            content: msg.content.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatGPTMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatGPTResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatGPTMessage,
}

impl ChatGPTClient {
    pub fn new(model: String) -> Result<Self, Box<dyn Error>> {
        let api_key = env::var("OPENAI_API_KEY")?;
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Ok(Self {
            api_key,
            model,
            client,
        })
    }

    pub async fn chat(
        &self,
        system_prompt: Option<String>,
        history: Vec<Message>,
        user_message: String,
    ) -> Result<Message, Box<dyn Error>> {
        let url = "https://api.openai.com/v1/chat/completions";

        let mut messages = Vec::new();

        if let Some(system) = system_prompt {
            messages.push(ChatGPTMessage {
                role: Role::System.as_str().to_string(),
                content: system,
            });
        }

        messages.extend(history.iter().map(ChatGPTMessage::from));

        messages.push(ChatGPTMessage {
            role: Role::User.as_str().to_string(),
            content: user_message,
        });

        let request = ChatRequest {
            model: self.model.clone(),
            messages,
        };

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json::<ChatGPTResponse>()
            .await?;

        response
            .choices
            .first()
            .map(|choice| choice.message.clone().into())
            .ok_or_else(|| "No response from ChatGPT".into())
    }
}

#[async_trait]
impl LLMClient for ChatGPTClient {
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
        let client = ChatGPTClient::new("gpt-4".to_string()).unwrap();

        let response = client
            .chat(None, vec![], "What is Rust?".to_string())
            .await
            .unwrap();

        assert!(!response.content.is_empty());
    }

    #[tokio::test]
    async fn test_chat_with_system_prompt() {
        let client = ChatGPTClient::new("gpt-4".to_string()).unwrap();

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
        let client = ChatGPTClient::new("gpt-4".to_string()).unwrap();

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
