use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use google_ai_rs::{Client, GenerativeModel};

const EXPENSIVE_MODEL: &str = "gemini-3.5-flash-lite";
const CHEAP_MODEL: &str = "gemini-3.5-flash-lite";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SceneOutline {
    id: String,
    title: String,
    #[serde(rename = "type")]
    scene_type: String, // "educational" or "narrative"
    setting: String, // "classroom" or "campus"
    summary: String,
    learning_objectives: Vec<String>,
    characters_present: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Storyboard {
    scenes: Vec<SceneOutline>,
}

async fn llm(client: &Client, model: &str, prompt: &str, system: &str) -> Result<String, Box<dyn Error>> {
    println!("\n--- Prompt ---\n{prompt}\n--------------");

    let mut gen_model: GenerativeModel = client.generative_model(model);
    if !system.is_empty() {
        gen_model = gen_model.with_system_instruction(system);
    }

    let response = gen_model.generate_content(prompt).await?;
    let text = response.text().to_string();
    Ok(text)
}

/// Helper to recursively copy directories
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

async fn run_pipeline(client: &Client, user_prompt: &str, base_dir: &Path) -> Result<(), Box<dyn Error>> {
    let char_info = fs::read_to_string(base_dir.join("data/character_info.txt"))?;
    
    // --- Step 1: Storyboard / Outline Generation ---
    println!("[1/4] Generating structured Storyboard Outline...");
    let storyboard_system = 
        "You are an expert visual novel designer and curriculum developer.\n\
         Generate a structured storyboard/outline for a visual novel in JSON format.\n\
         The JSON must match this exact schema:\n\
         {\n\
           \"scenes\": [\n\
             {\n\
               \"id\": \"scene_1_intro\",\n\
               \"title\": \"Introduction to Concepts\",\n\
               \"type\": \"educational\",\n\
               \"setting\": \"classroom\",\n\
               \"summary\": \"Ruby explains the core concept directly to the player. The player listens attentively. Ruby shows she likes the player.\",\n\
               \"learning_objectives\": [\"Objective 1\", \"Objective 2\"],\n\
               \"characters_present\": [\"Ruby\"]\n\
             }\n\
           ]\n\
         }\n\
         Design a story with 3 to 5 scenes containing a mix of educational core concepts and narrative B-plot character moments.\n\
         All narrative and educational summary descriptions must be from a first-person perspective ('I', 'me', 'my').\n\
         Output ONLY valid JSON. No markdown blocks, no commentary.";

    let storyboard_prompt = format!(
        "Educational Topic: {}\n\n\
         Character Info:\n{}\n\n\
         Generate the JSON storyboard now.",
        user_prompt, char_info
    );

    let raw_storyboard = llm(client, EXPENSIVE_MODEL, &storyboard_prompt, storyboard_system).await?;
    
    // Clean JSON markdown tags if present
    let cleaned_json = raw_storyboard
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string();

    let storyboard: Storyboard = serde_json::from_str(&cleaned_json)?;
    println!("[1/4] Storyboard successfully planned with {} scenes.", storyboard.scenes.len());

    // Create target game folder by copying our template project
    let output_game_dir = base_dir.join("output_game");
    if output_game_dir.exists() {
        let _ = fs::remove_dir_all(&output_game_dir);
    }
    copy_dir_all(base_dir.join("templates/template_project"), &output_game_dir)?;
    
    let scenes_dir = output_game_dir.join("game/scenes");
    fs::create_dir_all(&scenes_dir)?;

    // --- Step 2: Generate Each Scene Independently ---
    let mut prev_scene_summary: Option<String> = None;
    let mut scene_ids = Vec::new();

    for (index, scene) in storyboard.scenes.iter().enumerate() {
        println!(
            "[2/4] Generating Scene {}/{} [{}]: {}...",
            index + 1,
            storyboard.scenes.len(),
            scene.id,
            scene.title
        );

        let mut scene_prompt = format!(
            "Generate a Ren'Py scene script for: \"{}\".\n\
             Scene ID: {}\n\
             Type: {}\n\
             Setting: {}\n\
             Summary: {}\n\
             Learning Objectives: {:?}\n\
             Characters Present: {:?}\n\n\
             Character Profiles:\n{}\n\n",
            scene.title, scene.id, scene.scene_type, scene.setting, scene.summary, scene.learning_objectives, scene.characters_present, char_info
        );

        if let Some(prev) = &prev_scene_summary {
            scene_prompt.push_str(&format!("Previous Scene Summary (for continuity):\n{}\n\n", prev));
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

        let scene_script = llm(
            client,
            CHEAP_MODEL,
            &scene_prompt,
            "You are an expert Ren'Py writer. Write clean dialogue and sprite positions. Output ONLY the raw Ren'Py script for the requested label. No markdown code blocks."
        ).await?;

        // Clean any code block wrappers
        let clean_script = scene_script
            .trim()
            .trim_start_matches("```renpy")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string();

        let scene_file_name = format!("{}.rpy", scene.id);
        fs::write(scenes_dir.join(&scene_file_name), &clean_script)?;
        
        prev_scene_summary = Some(scene.summary.clone());
        scene_ids.push(scene.id.clone());
    }

    // --- Step 3: Assemble Main script.rpy ---
    println!("[3/4] Assembling script.rpy...");
    let mut script_content = String::new();
    script_content.push_str(
        "# Main script entry point.\n\
         # Automatically generated by Novelize modular pipeline.\n\n\
         label start:\n"
    );

    for (index, id) in scene_ids.iter().enumerate() {
        if index == 0 {
            script_content.push_str(&format!("    # Begin visual novel\n    call {}\n", id));
        } else {
            script_content.push_str(&format!("    call {}\n", id));
        }
    }
    script_content.push_str("    return\n");

    fs::write(output_game_dir.join("game/script.rpy"), script_content)?;

    // --- Step 4: Verification ---
    println!("[4/4] Project successfully modularized and built at: {}", output_game_dir.display());

    Ok(())
}

fn base_dir() -> Result<PathBuf, Box<dyn Error>> {
    // Locate the Cargo.toml workspace directory
    let mut dir = std::env::current_dir()?;
    while !dir.join("Cargo.toml").exists() {
        if let Some(parent) = dir.parent() {
            dir = parent.to_path_buf();
        } else {
            return Err("Could not find workspace root containing Cargo.toml".into());
        }
    }
    Ok(dir)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new(std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY unset")).await?;

    let base_dir = base_dir()?;
    let user_prompt = "How does concurrency work in Rust?";

    run_pipeline(&client, user_prompt, &base_dir).await?;

    Ok(())
}