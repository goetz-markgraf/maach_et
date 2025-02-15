use clap::Parser;
use std::error::Error;

use crate::llm_api::{ChatGPTClient, LLMClient, OllamaClient};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Model to use, in format provider/model (e.g. ollama/qwen2.5-coder or openai/gpt-4)
    #[arg(long, default_value = "ollama/qwen2.5-coder")]
    pub model: String,

    /// Hostname for Ollama server (ignored for OpenAI models)
    #[arg(long, default_value = "localhost")]
    pub hostname: String,

    /// Port for Ollama server (ignored for OpenAI models)
    #[arg(long, default_value_t = 11434)]
    pub port: u16,
}

impl Config {
    pub fn create_llm_client(&self) -> Result<Box<dyn LLMClient>, Box<dyn Error>> {
        let parts: Vec<&str> = self.model.split('/').collect();
        if parts.len() != 2 {
            return Err("Model must be in format provider/model".into());
        }

        let (provider, model) = (parts[0], parts[1]);

        match provider {
            "ollama" => Ok(Box::new(OllamaClient::new(
                self.hostname.clone(),
                self.port,
                model.to_string(),
            ))),
            "openai" => {
                let client = ChatGPTClient::new(model.to_string())?;
                Ok(Box::new(client))
            }
            _ => Err(format!("Unsupported provider: {}", provider).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_values() {
        let config = Config::parse_from(["test"]);
        assert_eq!(config.model, "ollama/qwen2.5-coder");
        assert_eq!(config.hostname, "localhost");
        assert_eq!(config.port, 11434);
    }

    #[test]
    fn test_custom_values() {
        let config = Config::parse_from([
            "test",
            "--model",
            "ollama/codellama",
            "--hostname",
            "api.example.com",
            "--port",
            "8080",
        ]);
        assert_eq!(config.model, "ollama/codellama");
        assert_eq!(config.hostname, "api.example.com");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_create_ollama_client() {
        let config = Config::parse_from([
            "test",
            "--model",
            "ollama/codellama",
            "--hostname",
            "test.local",
            "--port",
            "9999",
        ]);
        let client = config.create_llm_client().unwrap();
        // We can't directly test the client's internal state, but we can verify
        // it was created without error
    }

    #[test]
    fn test_create_openai_client() {
        env::set_var("OPENAI_API_KEY", "test-key");
        let config = Config::parse_from(["test", "--model", "openai/gpt-4"]);
        let client = config.create_llm_client().unwrap();
        // The hostname and port should be ignored for OpenAI
    }

    #[test]
    fn test_invalid_model_format() {
        let config = Config::parse_from(["test", "--model", "invalid-model"]);
        assert!(config.create_llm_client().is_err());
    }

    #[test]
    fn test_unsupported_provider() {
        let config = Config::parse_from(["test", "--model", "unsupported/model"]);
        assert!(config.create_llm_client().is_err());
    }

    #[test]
    fn test_openai_without_api_key() {
        env::remove_var("OPENAI_API_KEY");
        let config = Config::parse_from(["test", "--model", "openai/gpt-4"]);
        assert!(config.create_llm_client().is_err());
    }
}
