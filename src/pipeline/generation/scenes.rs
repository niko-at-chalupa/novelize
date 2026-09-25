use std::error::Error;

use super::super::etc::clean_code_block_wrappers;
use super::super::story::SceneOutline;
use super::{llm, ExtraContext};
use google_ai_rs::Client;

pub async fn generate_scene(
    model: &str,
    client: &Client,
    scene: &SceneOutline,
    prev_scene_summary: Option<String>,
    extra_context: &ExtraContext,
) -> Result<String, Box<dyn Error>> {
    let mut scene_prompt = format!(
        "Generate a Ren'Py scene script for: \"{}\".\n\
            Scene ID: {}\n\
            Type: {}\n\
            Setting: {}\n\
            Summary: {}\n\
            Learning Objectives: {:?}\n\
            Visual assets and naming:\n\
            {}\n\
            ---\
            {}
        ",
        scene.title,
        scene.id,
        scene.scene_type,
        scene.setting,
        scene.summary,
        scene.learning_objectives,
        extra_context.visuals_text(),
        extra_context.dialogue_text()
    );

    if let Some(prev) = &prev_scene_summary {
        scene_prompt.push_str(&format!(
            "Previous Scene Summary (for continuity):\n{}\n\n",
            prev
        ));
    }

    scene_prompt.push_str(&format!(
        "Output ONLY the Ren'Py code starting with `label <scene_id>:` and ending with `return`. \
            Do not wrap it in markdown code blocks. \
            The visual novel must be written entirely in the first-person perspective of the player ('I', 'me', 'my'). \
            Use MC (mc) as character short name for player's spoken dialogue. MC must never appear as a sprite on screen. \
            Use Ruby (r) as character short name. If other characters are introduced, define them with appropriate short names. \
            Show/hide sprites appropriately according to the template visual assets. \
            Template-specific scene instructions:\n\
            {}\
            Example:\n\
            label scene_1_intro:\n\
                scene bg classroom\n\
                show ruby school happy at left\n\
                \"I walk into the classroom, and Ruby smiles warmly at me.\"\n\
                r \"Hello! Ready to learn? Let's keep it clear and direct today, no silly metaphors.\"\n\
                mc \"Yes, thank you. Let's do it.\"\n\
                return",
            extra_context.scene_prompt
    ));

    let clean_script = {
        let scene_script = llm(
            client,
            model,
            &scene_prompt,
            &format!(
                "You are an expert Ren'Py writer. Write clean dialogue and sprite positions. Output ONLY the raw Ren'Py script for the requested label. No markdown code blocks.\n\nTemplate-specific scene system instructions:\n{}",
                extra_context.scene_system
            )
        ).await?;
        clean_code_block_wrappers(&scene_script)
    };
    Ok(clean_script)
}
