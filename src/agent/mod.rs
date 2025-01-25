use crate::llm_api::{Message, LLMAPI};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentResponse {
    Complete(String), // Task completed successfully with optional result message
    Reject(String),   // Task rejected with reason
    Partial(String),  // Partial completion with description of what's left
}

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub system_prompt: String,
    pub conversation_history: Vec<Message>,
}

impl AgentContext {
    pub fn new(system_prompt: String) -> Self {
        Self {
            system_prompt,
            conversation_history: Vec::new(),
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.conversation_history.push(message);
    }
}

#[async_trait]
pub trait Agent: Send + Sync {
    /// Returns the description of what this agent can do
    fn description(&self) -> &str;

    /// Process a task and return the result
    async fn process_task(
        &self,
        context: &mut AgentContext,
        task: &str,
    ) -> Result<AgentResponse, Box<dyn Error>>;
}

// Basic implementation of an agent that uses an LLM
pub struct BasicAgent {
    description: String,
    llm_api: LLMAPI,
}

impl BasicAgent {
    pub fn new(description: String, llm_api: LLMAPI) -> Self {
        Self {
            description,
            llm_api,
        }
    }
}

#[async_trait]
impl Agent for BasicAgent {
    fn description(&self) -> &str {
        &self.description
    }

    async fn process_task(
        &self,
        context: &mut AgentContext,
        task: &str,
    ) -> Result<AgentResponse, Box<dyn Error>> {
        // Add the task to the conversation history
        context.add_message(Message {
            role: "user".to_string(),
            content: task.to_string(),
        });

        // Get response from LLM
        let response = self
            .llm_api
            .chat(
                Some(context.system_prompt.clone()),
                context.conversation_history.clone(),
                task.to_string(),
            )
            .await?;

        // Add the response to the conversation history
        context.add_message(response.clone());

        // Parse the response to determine the agent's action
        // This is a simple implementation - in practice, you'd want to structure
        // the LLM's response more formally
        let content = response.content.to_lowercase();
        if content.contains("complete") {
            Ok(AgentResponse::Complete(response.content))
        } else if content.contains("reject") {
            Ok(AgentResponse::Reject(response.content))
        } else {
            Ok(AgentResponse::Partial(response.content))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm_api::LLMClient;

    struct MockLLMClient {
        response_type: String,
    }

    #[async_trait]
    impl LLMClient for MockLLMClient {
        async fn chat(
            &self,
            _system_prompt: Option<String>,
            _history: Vec<Message>,
            _user_message: String,
        ) -> Result<Message, Box<dyn Error>> {
            Ok(Message {
                role: "assistant".to_string(),
                content: format!("This is a {} response", self.response_type),
            })
        }
    }

    #[tokio::test]
    async fn test_basic_agent_complete() {
        let mock_client = MockLLMClient {
            response_type: "complete".to_string(),
        };
        let llm_api = LLMAPI::new(mock_client);
        let agent = BasicAgent::new("Test Agent".to_string(), llm_api);
        let mut context = AgentContext::new("You are a test agent.".to_string());

        let response = agent.process_task(&mut context, "Test task").await.unwrap();

        match response {
            AgentResponse::Complete(msg) => assert!(msg.contains("complete")),
            _ => panic!("Expected Complete response"),
        }

        assert_eq!(context.conversation_history.len(), 2);
    }

    #[tokio::test]
    async fn test_basic_agent_reject() {
        let mock_client = MockLLMClient {
            response_type: "reject".to_string(),
        };
        let llm_api = LLMAPI::new(mock_client);
        let agent = BasicAgent::new("Test Agent".to_string(), llm_api);
        let mut context = AgentContext::new("You are a test agent.".to_string());

        let response = agent.process_task(&mut context, "Test task").await.unwrap();

        match response {
            AgentResponse::Reject(msg) => assert!(msg.contains("reject")),
            _ => panic!("Expected Reject response"),
        }
    }

    #[tokio::test]
    async fn test_basic_agent_partial() {
        let mock_client = MockLLMClient {
            response_type: "partial".to_string(),
        };
        let llm_api = LLMAPI::new(mock_client);
        let agent = BasicAgent::new("Test Agent".to_string(), llm_api);
        let mut context = AgentContext::new("You are a test agent.".to_string());

        let response = agent.process_task(&mut context, "Test task").await.unwrap();

        match response {
            AgentResponse::Partial(msg) => assert!(msg.contains("partial")),
            _ => panic!("Expected Partial response"),
        }
    }

    #[test]
    fn test_agent_context() {
        let mut context = AgentContext::new("Test prompt".to_string());
        assert_eq!(context.conversation_history.len(), 0);

        context.add_message(Message {
            role: "user".to_string(),
            content: "Test message".to_string(),
        });
        assert_eq!(context.conversation_history.len(), 1);
    }
}
