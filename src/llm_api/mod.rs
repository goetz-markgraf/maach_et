use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

// Define the trait for LLM clients
#[async_trait::async_trait]
pub trait LLMClient: Send + Sync {
    async fn chat(
        &self,
        system_prompt: Option<String>,
        history: Vec<Message>,
        user_message: String,
    ) -> Result<Message, Box<dyn Error>>;
}

// Generic LLM API wrapper
pub struct LLMAPI {
    client: Box<dyn LLMClient>,
}

impl LLMAPI {
    pub fn new<T: LLMClient + 'static>(client: T) -> Self {
        Self {
            client: Box::new(client),
        }
    }

    pub async fn chat(
        &self,
        system_prompt: Option<String>,
        history: Vec<Message>,
        user_message: String,
    ) -> Result<Message, Box<dyn Error>> {
        self.client.chat(system_prompt, history, user_message).await
    }
}

pub mod chat_gpt;
pub mod ollama;

// Re-export implementations
#[allow(unused_imports)]
pub use chat_gpt::ChatGPTClient;
#[allow(unused_imports)]
pub use ollama::OllamaClient;

#[cfg(test)]
mod tests {
    use super::*;

    // Mock LLM client for testing
    struct MockLLMClient;

    #[async_trait::async_trait]
    impl LLMClient for MockLLMClient {
        async fn chat(
            &self,
            system_prompt: Option<String>,
            history: Vec<Message>,
            user_message: String,
        ) -> Result<Message, Box<dyn Error>> {
            Ok(Message {
                role: "assistant".to_string(),
                content: format!(
                    "Mock response to: {}. System prompt: {:?}, History length: {}",
                    user_message,
                    system_prompt,
                    history.len()
                ),
            })
        }
    }

    #[tokio::test]
    async fn test_llm_api_with_ollama() {
        let ollama = OllamaClient::new("localhost".to_string(), 11434, "codellama".to_string());
        let api = LLMAPI::new(ollama);

        let response = api
            .chat(
                Some("You are a test assistant.".to_string()),
                vec![],
                "Test message".to_string(),
            )
            .await
            .unwrap();

        assert!(!response.content.is_empty());
    }

    #[tokio::test]
    async fn test_llm_api_with_chatgpt() {
        if let Ok(chatgpt) = ChatGPTClient::new("gpt-4".to_string()) {
            let api = LLMAPI::new(chatgpt);

            let response = api
                .chat(
                    Some("You are a test assistant.".to_string()),
                    vec![],
                    "Test message".to_string(),
                )
                .await
                .unwrap();

            assert!(!response.content.is_empty());
        }
    }

    #[tokio::test]
    async fn test_llm_api_with_mock() {
        let mock_client = MockLLMClient;
        let api = LLMAPI::new(mock_client);

        let history = vec![
            Message {
                role: "user".to_string(),
                content: "Previous message".to_string(),
            },
            Message {
                role: "assistant".to_string(),
                content: "Previous response".to_string(),
            },
        ];

        let response = api
            .chat(
                Some("System prompt".to_string()),
                history,
                "Test message".to_string(),
            )
            .await
            .unwrap();

        assert!(response.content.contains("Test message"));
        assert!(response.content.contains("System prompt"));
        assert!(response.content.contains("History length: 2"));
    }

    #[tokio::test]
    async fn test_conversation_flow() {
        let mock_client = MockLLMClient;
        let api = LLMAPI::new(mock_client);

        // First message
        let response1 = api
            .chat(None, vec![], "First message".to_string())
            .await
            .unwrap();

        // Second message with history
        let history = vec![
            Message {
                role: "user".to_string(),
                content: "First message".to_string(),
            },
            response1,
        ];

        let response2 = api
            .chat(None, history, "Second message".to_string())
            .await
            .unwrap();

        assert!(response2.content.contains("Second message"));
        assert!(response2.content.contains("History length: 2"));
    }
}
