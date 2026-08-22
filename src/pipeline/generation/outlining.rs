use std::error::Error;
use super::super::etc::clean_code_block_wrappers;
use super::super::story::Storyboard;
use google_ai_rs::Client;
use super::llm;

pub async fn generate_storyboard(model: &str, client: &Client, topic: &str, char_info: &str) -> Result<Storyboard, Box<dyn Error>> {
    let storyboard_system = r#"You are an expert visual novel designer and curriculum developer.
         Generate a structured storyboard/outline for a visual novel in JSON format.
         The JSON must match this exact schema:
         {
           \"scenes\": [
             {
               "id": "scene_1_intro",
               "title": "Introduction to Concepts",
               "type": "educational\",
               "setting": "classroom",
               "summary": "Ruby explains the core concept directly to the player. The player listens attentively. Ruby shows she likes the player.",
               "learning_objectives": ["Objective 1", "Objective 2"],
               "characters_present": ["Ruby"]
             }
           ]
         }
         Design a story with 3 to 5 scenes containing a mix of educational core concepts and narrative B-plot character moments.
         All narrative and educational summary descriptions must be from a first-person perspective ('I', 'me', 'my').
         Output ONLY valid JSON. No markdown blocks, no commentary."#;

    let storyboard_prompt = format!(
        r#"Educational Topic: {}
         Character Info: {}
         Generate the JSON storyboard now."#,
        topic, char_info
    );

    let cleaned_json = {
        let raw_storyboard = llm(
            client,
            model,
            &storyboard_prompt,
            storyboard_system,
        )
        .await?;
        clean_code_block_wrappers(&raw_storyboard)
    };

    let storyboard: Storyboard = serde_json::from_str(&cleaned_json)?;
    Ok(storyboard)
}