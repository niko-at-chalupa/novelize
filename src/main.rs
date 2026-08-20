mod renpy;
use google_ai_rs::Client;
mod llm;
mod story;
mod pipeline;
use std::path::PathBuf;
use clap::Parser;
use renpy::is_valid_renpy_sdk;
use std::error::Error;
use pipeline::run_pipeline;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    topic: String,

    #[arg(short, long)]
    renpy_sdk: PathBuf,

    #[arg(short, long, default_value = "output_game")]
    output_game_dir: PathBuf,
}

fn base_dir() -> Result<PathBuf, Box<dyn Error>> {
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
    let client =
        Client::new(std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY unset")).await?;

    let base_dir = base_dir()?;

    let args = Args::parse();

    if args.output_game_dir.exists() {
        panic!("output_game_dir already exists")
    }

    if !is_valid_renpy_sdk(&args.renpy_sdk) {
        panic!("renpy sdk provided invalid")
    }
    
    run_pipeline(&client, &args.topic, &base_dir, &args.output_game_dir).await?;

    Ok(())
}
