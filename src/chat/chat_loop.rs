use std::io::{self, Write};

use crate::llm_api::{LLMClient, Message, Role};

pub struct ChatLoop {
    llm_api: Box<dyn LLMClient>,
    system_prompt: String,
    conversation_history: Vec<Message>,
}

impl ChatLoop {
    pub fn new(llm_api: Box<dyn LLMClient>, system_prompt: String) -> Self {
        let conversation_history = vec![];

        Self {
            llm_api,
            system_prompt,
            conversation_history,
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        loop {
            print!("/USER/ ");
            io::stdout().flush()?;

            let mut user_input = String::new();
            io::stdin().read_line(&mut user_input)?;
            let user_input = user_input.trim();

            // Check for exit commands
            if matches!(user_input, "/bye" | "/exit" | "/quit") {
                println!("Goodbye!");
                break;
            }

            // Add user message to history
            self.conversation_history.push(Message {
                role: Role::User,
                content: user_input.to_string(),
            });

            // Get response from LLM
            match self
                .llm_api
                .chat(
                    Some(self.system_prompt.clone()),
                    self.conversation_history.clone(),
                    user_input.to_string(),
                )
                .await
            {
                Ok(response) => {
                    println!("/ASSISTANT/ {}", response.content);
                    self.conversation_history.push(response);
                }
                Err(e) => {
                    eprintln!("Error getting response: {}", e);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use tokio::sync::Mutex;

    use super::*;
    use std::{error::Error, sync::Arc};

    struct MockLLMAPI {
        responses: Arc<Mutex<Vec<String>>>,
    }

    impl MockLLMAPI {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses)),
            }
        }
    }

    #[async_trait]
    impl LLMClient for MockLLMAPI {
        async fn chat(
            &self,
            _system_prompt: Option<String>,
            _history: Vec<Message>,
            _user_message: String,
        ) -> Result<Message, Box<dyn Error>> {
            let mut responses = self.responses.lock().await;
            if responses.is_empty() {
                return Err("No more mock responses".into());
            }

            let response = responses.remove(0);
            Ok(Message {
                role: Role::Agent,
                content: response,
            })
        }
    }

    #[tokio::test]
    async fn test_chat_loop_initialization() {
        let mock_api = Box::new(MockLLMAPI::new(vec![]));
        let system_prompt = "Test system prompt".to_string();
        let chat_loop = ChatLoop::new(mock_api, system_prompt);

        // Test that the initial conversation history contains the system prompt
        assert_eq!(chat_loop.conversation_history.len(), 0);
    }

    #[tokio::test]
    async fn test_chat_loop_conversation() {
        let mock_responses = vec![
            "Hello! How can I help?".to_string(),
            "That's interesting!".to_string(),
        ];
        let mock_api = Box::new(MockLLMAPI::new(mock_responses));
        let system_prompt = "Test system prompt".to_string();
        let mut chat_loop = ChatLoop::new(mock_api, system_prompt.clone());

        // Simulate adding a user message
        chat_loop.conversation_history.push(Message {
            role: Role::User,
            content: "Hi there!".to_string(),
        });

        // Test getting a response
        let response = chat_loop
            .llm_api
            .chat(
                Some(system_prompt),
                chat_loop.conversation_history.clone(),
                "Hi there!".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(response.role, Role::Agent);
        assert_eq!(response.content, "Hello! How can I help?");

        // Add the response to history
        chat_loop.conversation_history.push(response);

        // Verify conversation history
        assert_eq!(chat_loop.conversation_history.len(), 2);
        assert_eq!(chat_loop.conversation_history[0].role, Role::User);
        assert_eq!(chat_loop.conversation_history[1].role, Role::Agent);
    }

    #[tokio::test]
    async fn test_chat_loop_error_handling() {
        let mock_api = Box::new(MockLLMAPI::new(vec![])); // Empty responses will trigger an error
        let system_prompt = "Test system prompt".to_string();
        let chat_loop = ChatLoop::new(mock_api, system_prompt.clone());

        // Test error case
        let result = chat_loop
            .llm_api
            .chat(
                Some(system_prompt),
                chat_loop.conversation_history.clone(),
                "Hi there!".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "No more mock responses");
    }
}
