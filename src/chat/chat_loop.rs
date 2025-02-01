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

    pub fn get_conversation_history(&self) -> Vec<Message> {
        self.conversation_history.clone()
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

            println!("Thinking...");

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
                    self.conversation_history.push(Message {
                        role: Role::User,
                        content: user_input.to_string(),
                    });
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
}
