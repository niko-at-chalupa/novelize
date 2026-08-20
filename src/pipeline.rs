use crate::llm::llm;
use std::error::Error;
use std::fs;
use std::path::Path;
use crate::story::Storyboard;
use google_ai_rs::Client;
use regex::Regex;
use crate::renpy::run_renpy_lint;

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

fn clean_code_block_wrappers(text: &String) -> String {
    let re = Regex::new(r"(?m)^```[^\r\n]*\r?\n?").unwrap();
    re.replace(&text, "")
        .into_owned()
        .trim_end_matches("```")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_code_block_wrappers_test() {
        let content = r#"iajsfiajisfjajfiajsifjija
        asfiajsifjasfjiajsfjasifjaisf
        asfiajisfajifsjiaifjiajsfijs
        asdjasidjijd"#;
        let prefixes = vec!["```renpy", "```json", "```"];
        let end = "```";

        let mut samples: Vec<String> = vec![];

        // {prefix}
        // {content}
        // {end}
        for prefix in prefixes {
            samples.push(format!("{}\n{}\n{}", prefix, content, end));
        }
        // {content}
        samples.push(content.to_string());

        for sample in samples {
            let cleaned = clean_code_block_wrappers(&sample);
            assert_eq!(cleaned, content)
        }
    }
}


const EXPENSIVE_MODEL: &str = "gemini-3.5-flash-lite";
const CHEAP_MODEL: &str = "gemini-3.5-flash-lite";

#[derive(serde::Deserialize)]
struct LintTriage {
    affected_scene_ids: Vec<String>,
}

async fn triage_lint_errors(
    client: &Client,
    lint_report: &str,
    known_scene_ids: &[String],
) -> Result<Vec<String>, Box<dyn Error>> {
    let system = "You are a Ren'Py build error triager.\n\
        Given a raw `renpy lint` report and a list of known scene label ids, \
        identify which scene ids the errors belong to.\n\
        Output ONLY valid JSON matching this schema, no markdown, no commentary:\n\
        { \"affected_scene_ids\": [\"scene_1_intro\"] }\n\
        Only include ids from the known list. If an error can't be attributed to a specific \
        scene (e.g. a global script.rpy issue), omit it.";

    let prompt = format!(
        "Known scene ids: {:?}\n\nLint report:\n{}\n\nWhich scene ids need fixing?",
        known_scene_ids, lint_report
    );

    let raw = llm(client, CHEAP_MODEL, &prompt, system).await?;
    let cleaned = clean_code_block_wrappers(&raw);
    let triage: LintTriage = serde_json::from_str(&cleaned)?;
    Ok(triage.affected_scene_ids)
}

async fn fix_scene_file(
    client: &Client,
    scenes_dir: &Path,
    scene_id: &str,
    lint_report: &str,
) -> Result<(), Box<dyn Error>> {
    let file_path = scenes_dir.join(format!("{}.rpy", scene_id));
    let current_script = fs::read_to_string(&file_path)?;

    let system = "You are an expert Ren'Py debugger.\n\
        You will be given a broken Ren'Py scene script and the full lint report for the project.\n\
        Fix ONLY the errors in this script that are attributable to it (e.g. undefined images, \
        malformed tags, bad label/menu syntax, unclosed text tags).\n\
        Preserve the dialogue, structure, characters, and intent exactly — do not rewrite content \
        that isn't broken.\n\
        Output ONLY the corrected raw Ren'Py script for this label, no markdown code blocks, \
        no commentary.";

    let prompt = format!(
        "Scene id: {}\n\n\
         Current script:\n{}\n\n\
         Full project lint report (only fix parts relevant to this scene):\n{}",
        scene_id, current_script, lint_report
    );

    let raw_fixed = llm(client, CHEAP_MODEL, &prompt, system).await?;
    let fixed_script = clean_code_block_wrappers(&raw_fixed);
    fs::write(&file_path, fixed_script)?;

    Ok(())
}

pub async fn run_pipeline(
    client: &Client,
    user_prompt: &str,
    base_dir: &Path,
    output_game_dir: &Path,
    max_fix_attempts: u8,
    sdk_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let char_info = fs::read_to_string(base_dir.join("data/character_info.txt"))?;

    println!("[1/4] Generating structured Storyboard Outline...");
    let storyboard_system = "You are an expert visual novel designer and curriculum developer.\n\
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

    let cleaned_json = {
        let raw_storyboard = llm(
            client,
            EXPENSIVE_MODEL,
            &storyboard_prompt,
            storyboard_system,
        )
        .await?;
        clean_code_block_wrappers(&raw_storyboard)
    };

    let storyboard: Storyboard = serde_json::from_str(&cleaned_json)?;
    println!(
        "[1/4] Storyboard successfully planned with {} scenes.",
        storyboard.scenes.len()
    );

    // Create target game folder by copying our template project
    if output_game_dir.exists() {
        let _ = fs::remove_dir_all(&output_game_dir);
    }
    copy_dir_all(
        base_dir.join("templates/template_project"),
        &output_game_dir,
    )?;

    let scenes_dir = output_game_dir.join("game/scenes");
    fs::create_dir_all(&scenes_dir)?;

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
                CHEAP_MODEL,
                &scene_prompt,
                "You are an expert Ren'Py writer. Write clean dialogue and sprite positions. Output ONLY the raw Ren'Py script for the requested label. No markdown code blocks."
            ).await?;
            clean_code_block_wrappers(&scene_script)
        };

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
         label start:\n",
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

    // --- Step 4: Verification (and linting) ---
    println!("[4/4] Verifying generated project with Ren'Py lint...");

    for attempt in 1..=max_fix_attempts {
        let lint_output = run_renpy_lint(&sdk_path, &output_game_dir)?;
        if lint_output.status.success() {
            println!("[4/4] Lint passed on attempt {}.", attempt);
            break;
        }

        if attempt == max_fix_attempts {
            return Err(format!(
                "Ren'Py lint still failing after {} fix attempts:\n{}",
                max_fix_attempts,
                String::from_utf8_lossy(&lint_output.stdout)
            )
            .into());
        }

        let lint_report = format!(
            "{}\n{}",
            String::from_utf8_lossy(&lint_output.stdout),
            String::from_utf8_lossy(&lint_output.stderr)
        );

        println!(
            "[4/4] Lint failed (attempt {}/{}). Diagnosing affected scenes...",
            attempt, max_fix_attempts
        );

        let affected = triage_lint_errors(client, &lint_report, &scene_ids).await?;
        if affected.is_empty() {
            return Err(format!(
                "Lint failed but triage identified no fixable scene files:\n{}",
                lint_report
            )
            .into());
        }

        for scene_id in &affected {
            println!("[4/4]   Repairing scene: {}", scene_id);
            fix_scene_file(client, &scenes_dir, scene_id, &lint_report).await?;
        }
    }

    Ok(())
}