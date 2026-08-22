use std::error::Error;

use super::super::etc::clean_code_block_wrappers;
use super::super::story::SceneOutline;
use super::llm;
use google_ai_rs::Client;

pub async fn generate_scene(
    model: &str,
    client: &Client,
    scene: &SceneOutline,
    prev_scene_summary: Option<String>,
    char_info: &str,
) -> Result<String, Box<dyn Error>> {
    let mut scene_prompt = format!(
        "Generate a Ren'Py scene script for: \"{}\".\n\
            Scene ID: {}\n\
            Type: {}\n\
            Setting: {}\n\
            Summary: {}\n\
            Learning Objectives: {:?}\n\
            Characters Present: {:?}\n\n\
            Character Profiles:\n{}\n\n",
        scene.title,
        scene.id,
        scene.scene_type,
        scene.setting,
        scene.summary,
        scene.learning_objectives,
        scene.characters_present,
        char_info
    );

    if let Some(prev) = &prev_scene_summary {
        scene_prompt.push_str(&format!(
            "Previous Scene Summary (for continuity):\n{}\n\n",
            prev
        ));
    }

    scene_prompt.push_str(
        "Output ONLY the Ren'Py code starting with `label <scene_id>:` and ending with `return`. \
            Do not wrap it in markdown code blocks. \
            The visual novel must be written entirely in the first-person perspective of the player ('I', 'me', 'my'). \
            Use MC (mc) as character short name for player's spoken dialogue. MC must never appear as a sprite on screen. \
            Use Ruby (r) as character short name. If other characters are introduced, define them with appropriate short names. \
            Show/hide sprites appropriately: \
            - Ruby sprites: ruby school, ruby school happy, ruby school sad, ruby school flustered \
            - Backgrounds: bg classroom, bg campus \
            - Ruby traits: she likes the player, does NOT like metaphors and analogies, does NOT like misbehaved or rude people. \
            \n\
            You can use kinetic text tags to animate text in character dialogue or narration. You must use them EXTREMELY sparingly (no more than once or twice in the entire scene), and ONLY to emphasize strong, specific emotions (such as excitement, panic, anger, or extreme nervousness) on single words or short phrases: \
            - {bt=10}bouncing text{/bt}: Bounces text up/down (great for excitement, joy, or laughing). \
            - {sc=3}shaky text{/sc}: Shakes text in place (great for anger, nervousness, fear, or panic). \
            - {rotat=300}rotating text{/rotat}: Rotates text. \
            - {chaos}chaos text{/chaos}: Randomizes fonts/colors/sizes per frame (good for madness/confusion). \
            - {fi=0-3.0-50}fade-in text{/fi}: Fades/slides in character-by-character (good for slow, dramatic delivery). \
            - {swap=WordA@WordB@0.5}WordA{/swap}: Swaps between WordA and WordB every 0.5s. \
            \n\
            For educational scenes explaining programming, you also have a code block formatting tag: \
            - {code}your code here{/code}: Renders the text in a monospaced font with syntax highlighting (perfect for displaying brief code snippets or expressions). \
            \n\
            Example:\n\
            label scene_1_intro:\n\
                scene bg classroom\n\
                show ruby school happy at left\n\
                \"I walk into the classroom, and Ruby smiles warmly at me.\"\n\
                r \"Hello! Ready to learn? Let's keep it {bt=8}clear and direct{/bt} today, no silly metaphors.\"\n\
                mc \"{sc=2}Yes, thank you.{/sc} Let's do it.\"\n\
                return"
    );

    let clean_script = {
        let scene_script = llm(
            client,
            model,
            &scene_prompt,
            "You are an expert Ren'Py writer. Write clean dialogue and sprite positions. Output ONLY the raw Ren'Py script for the requested label. No markdown code blocks."
        ).await?;
        clean_code_block_wrappers(&scene_script)
    };
    Ok(clean_script)
}
