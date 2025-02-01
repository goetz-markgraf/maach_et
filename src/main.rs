use mach_et::chat::ChatLoop;
use mach_et::llm_api::OllamaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let llm_client = OllamaClient::new("localhost".to_string(), 11434, "deepseek-r1".to_string());

    let system_prompt =
        "You are a helpful assistant. Anwer each question as precise and as consise as possible"
            .to_string();

    let mut chat_loop = ChatLoop::new(Box::new(llm_client), system_prompt);

    chat_loop.run().await?;

    // print the length of the conversation history
    println!(
        "Conversation history length: {}",
        chat_loop.get_conversation_history().len()
    );

    Ok(())
}
