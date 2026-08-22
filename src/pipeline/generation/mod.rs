pub mod linting;
pub mod outlining;
pub mod scenes;

use google_ai_rs::{Client, GenerativeModel};
use std::error::Error;

pub async fn llm(
    client: &Client,
    model: &str,
    prompt: &str,
    system: &str,
) -> Result<String, Box<dyn Error>> {
    println!("\n--- Prompt ---\n{prompt}\n--------------");

    let mut gen_model: GenerativeModel = client.generative_model(model);
    if !system.is_empty() {
        gen_model = gen_model.with_system_instruction(system);
    }

    let response = gen_model.generate_content(prompt).await?;
    let text = response.text().to_string();
    Ok(text)
}
