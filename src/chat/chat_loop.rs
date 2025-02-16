use std::error::Error;
use std::io::{self, Write};

use crate::{
    chat::{git, tool_checker::run_tools},
    llm_api::{LLMClient, Message, Role},
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum LoopStatus {
    UserInput,
    ToolInput,
    Exit,
    Error,
}
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

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Check for uncommitted git changes
        if let Some((has_changes, files)) = git::has_uncommitted_changes() {
            if has_changes {
                println!("The following files have uncommitted changes:");
                for file in &files {
                    println!("  - {}", file);
                }
                print!("\nWould you like to commit these changes? (y: commit, n: proceed without committing, x: exit) ");
                io::stdout().flush()?;

                let mut response = String::new();
                io::stdin().read_line(&mut response)?;

                match response.trim().to_lowercase().as_str() {
                    "y" => {
                        print!("Enter commit message: ");
                        io::stdout().flush()?;

                        let mut commit_msg = String::new();
                        io::stdin().read_line(&mut commit_msg)?;

                        if let Err(e) = git::commit_all_changes(commit_msg.trim()) {
                            eprintln!("Failed to commit changes: {}", e);
                        } else {
                            println!("Changes committed successfully!");
                        }
                    }
                    "x" => {
                        println!("Exiting due to uncommitted changes.");
                        return Ok(());
                    }
                    "n" => {
                        println!("Proceeding without committing changes.");
                    }
                    _ => {
                        println!("Invalid option. Exiting.");
                        return Ok(());
                    }
                }
            }
        }

        let mut loop_status = LoopStatus::UserInput;
        let mut tool_input = String::new();

        while loop_status != LoopStatus::Exit && loop_status != LoopStatus::Error {
            let input = if loop_status == LoopStatus::UserInput {
                print!("/USER/ ");
                io::stdout().flush()?;

                let mut user_input = String::new();
                io::stdin().read_line(&mut user_input)?;

                // Check for exit commands
                if user_input.trim() == "/bye"
                    || user_input.trim() == "/exit"
                    || user_input.trim() == "/quit"
                {
                    println!("Goodbye!");
                    loop_status = LoopStatus::Exit;
                    continue;
                }
                format!("Help me with my task. {}\nKeep in mind to use the tools described in the system prompt", user_input.trim())
            } else {
                tool_input.clone()
            };

            println!("Thinking...");

            // Get response from LLM
            match self
                .llm_api
                .chat(
                    Some(self.system_prompt.clone()),
                    self.conversation_history.clone(),
                    input.to_string(),
                )
                .await
            {
                Ok(response) => {
                    println!("/ASSISTANT/ {}", response.content);
                    let response_str = response.content.clone();
                    self.conversation_history.push(Message {
                        role: Role::User,
                        content: input.to_string(),
                    });
                    self.conversation_history.push(response);

                    match run_tools(&response_str)? {
                        None => loop_status = LoopStatus::UserInput,
                        Some(tool_output) => {
                            tool_input = tool_output;
                            loop_status = LoopStatus::ToolInput;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error getting response: {}", e);
                    loop_status = LoopStatus::Error;
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
