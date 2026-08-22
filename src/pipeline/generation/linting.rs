use super::super::clean_code_block_wrappers;
use super::llm;
use google_ai_rs::Client;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(serde::Deserialize)]
struct LintTriage {
    affected_scene_ids: Vec<String>,
}

pub async fn triage_lint_errors(
    model: &str,
    client: &Client,
    lint_report: &str,
    known_scene_ids: &[String],
) -> Result<Vec<String>, Box<dyn Error>> {
    let system = r#"You are a Ren'Py build error triager.
Given a raw `renpy lint` report and a list of known scene label ids, identify which scene ids the errors belong to.
Output ONLY valid JSON matching this schema, no markdown, no commentary:
{ "affected_scene_ids": ["scene_1_intro"] }
Only include ids from the known list. If an error can't be attributed to a specific scene (e.g. a global script.rpy issue), omit it."#;

    let prompt = format!(
        r#"Known scene ids: {:?}

Lint report:
{}

Which scene ids need fixing?"#,
        known_scene_ids, lint_report
    );

    let raw = llm(client, model, &prompt, system).await?;
    let cleaned = clean_code_block_wrappers(&raw);
    let triage: LintTriage = serde_json::from_str(&cleaned)?;
    Ok(triage.affected_scene_ids)
}

pub async fn fix_scene_file(
    model: &str,
    client: &Client,
    scenes_dir: &Path,
    scene_id: &str,
    lint_report: &str,
) -> Result<(), Box<dyn Error>> {
    let file_path = scenes_dir.join(format!(r#"{}.rpy"#, scene_id));
    let current_script = fs::read_to_string(&file_path)?;

    let system = r#"You are an expert Ren'Py debugger.
You will be given a broken Ren'Py scene script and the full lint report for the project.
Fix ONLY the errors in this script that are attributable to it (e.g. undefined images, malformed tags, bad label/menu syntax, unclosed text tags).
Preserve the dialogue, structure, characters, and intent exactly — do not rewrite content that isn't broken.
Output ONLY the corrected raw Ren'Py script for this label, no markdown code blocks, no commentary."#;

    let prompt = format!(
        r#"Scene id: {}

Current script:
{}

Full project lint report (only fix parts relevant to this scene):
{}"#,
        scene_id, current_script, lint_report
    );

    let raw_fixed = llm(client, model, &prompt, system).await?;
    let fixed_script = clean_code_block_wrappers(&raw_fixed);
    fs::write(&file_path, fixed_script)?;

    Ok(())
}
