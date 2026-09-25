use super::super::etc::clean_code_block_wrappers;
use super::super::story::Storyboard;
use super::{llm, ExtraContext};
use google_ai_rs::Client;
use std::error::Error;

pub async fn generate_storyboard(
    model: &str,
    client: &Client,
    topic: &str,
    extra_context: &ExtraContext,
    num_scenes: u8,
) -> Result<Storyboard, Box<dyn Error>> {
    let storyboard_system = format!(
        r#"You are an expert visual novel designer and curriculum developer.
Generate a structured storyboard/outline for a visual novel in JSON format.
The JSON must match this exact schema:
{{
  "scenes": [
    {{
      "id": "scene_1_intro",
      "title": "Introduction to Concepts",
      "type": "educational",
      "setting": "classroom",
      "summary": "Ruby explains the core concept directly to the player. The player listens attentively. Ruby shows she likes the player.",
      "learning_objectives": ["Objective 1", "Objective 2"],
    }}
  ]
}}
Design a story with {} scenes containing a mix of educational core concepts and narrative B-plot character moments.
All narrative and educational summary descriptions must be from a first-person perspective ('I', 'me', 'my').
Output ONLY valid JSON. No markdown blocks, no commentary. Do NOT introduce characters that are not listed in the character info.

Character Info:
  {}

Template-specific storyboard instructions:
{}"#,
        num_scenes,
    extra_context.narrative_text(),
    extra_context.storyboard_system
    );

    let storyboard_prompt = format!(
        r#"Educational Topic: {}
      Character Info: {}
      Visual assets and naming:
      {}
      Template-specific storyboard prompt instructions:
      {}
Generate the JSON storyboard now."#,
        topic,
        extra_context.narrative_text(),
        extra_context.visuals_text(),
        extra_context.storyboard_prompt
    );

    let cleaned_json = {
        let raw_storyboard = llm(client, model, &storyboard_prompt, &storyboard_system).await?;
        clean_code_block_wrappers(&raw_storyboard)
    };

    let storyboard: Storyboard = serde_json::from_str(&cleaned_json)?;
    Ok(storyboard)
}