use mach_et::chat::{get_system_prompt, ChatLoop};
use mach_et::llm_api::OllamaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let llm_client = OllamaClient::new("localhost".to_string(), 11434, "qwen2.5-coder".to_string());

    let mut system_prompt = get_system_prompt();
    system_prompt.push_str(&mach_et::tools::get_tool_prompt());

    let mut chat_loop = ChatLoop::new(Box::new(llm_client), system_prompt);

    chat_loop.run().await?;

    // print the length of the conversation history
    println!(
        "Conversation history length: {}",
        chat_loop.get_conversation_history().len()
    );

    Ok(())
}
