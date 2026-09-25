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
    tracing::debug!("\n--- Prompt ---\n{prompt}\n--------------");

    let mut gen_model: GenerativeModel = client.generative_model(model);
    if !system.is_empty() {
        gen_model = gen_model.with_system_instruction(system);
    }

    let response = gen_model.generate_content(prompt).await?;
    let text = response.text().to_string();
    Ok(text)
}

/// Context loaded from the template for the two generation stages.
#[derive(Default)]
pub struct ExtraContext {
    pub narrative: Vec<String>,
    pub dialogue: Vec<String>,
    pub visuals: Vec<String>,
    pub storyboard_system: String,
    pub storyboard_prompt: String,
    pub scene_system: String,
    pub scene_prompt: String,
}

impl ExtraContext {
    pub fn narrative_text(&self) -> String {
        join_context(&self.narrative)
    }

    pub fn dialogue_text(&self) -> String {
        join_context(&self.dialogue)
    }

    pub fn visuals_text(&self) -> String {
        join_context(&self.visuals)
    }
}

fn join_context(contexts: &[String]) -> String {
    contexts
        .iter()
        .map(|context| format!("\n{context}"))
        .collect()
}