use clap::Parser;
use maach_et::chat::{get_system_prompt, ChatLoop};
use maach_et::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();
    let llm_client = config.create_llm_client()?;

    // Print model information
    let model_parts: Vec<&str> = config.model.split('/').collect();
    println!("Using LLM Provider: {}", model_parts[0]);
    println!("Using Model: {}", model_parts[1]);

    let mut system_prompt = get_system_prompt();
    system_prompt.push_str(&maach_et::tools::get_tool_prompt());

    let mut chat_loop = ChatLoop::new(llm_client, system_prompt);

    chat_loop.run().await?;

    // print the length of the conversation history
    println!(
        "Conversation history length: {}",
        chat_loop.get_conversation_history().len()
    );

    Ok(())
}
