mod renpy;
use dotenvy::dotenv;
use google_ai_rs::Client;
mod pipeline;
use clap::Parser;
use pipeline::run_pipeline;
use renpy::is_valid_renpy_sdk;
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    topic: String,

    #[arg(short, long)]
    renpy_sdk: PathBuf,

    #[arg(short, long, default_value = "output_game")]
    output_game_dir: PathBuf,

    #[arg(short, long, default_value_t = 3)]
    max_fix_attempts: u8,

    #[arg(short, long, default_value_t = 4)]
    num_scenes: u8,
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
    if let Err(_) = dotenv() {
        tracing::warn!("no env file found...");
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

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

    run_pipeline(
        &client,
        &args.topic,
        &base_dir,
        &args.output_game_dir,
        args.max_fix_attempts,
        &args.renpy_sdk,
        args.num_scenes,
    )
    .await?;

    Ok(())
}
