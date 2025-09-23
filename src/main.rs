use std::{
    env::{current_exe, var},
    fs::{File, create_dir_all, read_to_string},
    io::{Read, Write, stdin},
};

use anyhow::{Result, anyhow, bail};
use clap::{Parser, Subcommand};
use daemonize::Daemonize;
use git2::Repository;
use serde_json::{Value, from_str, json, to_string_pretty};

mod commit_message_generator;
mod committer;
mod git_ops;
mod types;

use commit_message_generator::CommitMessageGenerator;
use committer::Committer;

use crate::types::HookEvent;

/// Command line arguments for the auto-commit application
#[derive(Parser)]
#[clap(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Language to use for commit messages (for legacy compatibility)
    #[arg(short, long, default_value = "Japanese", env = "CC_AUTO_COMMIT_LANGUAGE")]
    pub language: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a hook configuration to <repository_root>/.claude/settings.local.json
    Install,
}

fn main() -> Result<()> {
    // Prevent recursive calls
    if var("CLAUDE_AUTO_COMMIT_RUNNING").is_ok() {
        return Ok(());
    }

    let args = Args::parse();

    match args.command {
        Some(Commands::Install) => install_hook(&args.language),
        None => {
            // Default behavior - run as a hook or commit message generator
            let mut input = String::new();
            stdin().read_to_string(&mut input)?;

            match from_str::<HookEvent>(&input) {
                Ok(hook_event) => {
                    match Daemonize::new()
                        .working_directory(hook_event.cwd())
                        .umask(0o027)
                        .start()
                    {
                        Ok(_) => Committer::new().handle_event(hook_event, &args.language),
                        Err(e) => bail!("Error starting daemon: {e}"),
                    }
                }
                Err(_) => {
                    // If the input is not a valid HookEvent, assume it's a diff content and
                    // generate a commit message from it.
                    println!("{}", CommitMessageGenerator::new(&args.language)?.generate(&input));
                    Ok(())
                }
            }
        }
    }
}

fn install_hook(language: &str) -> Result<()> {
    let repo_root = Repository::discover(".")?
        .workdir()
        .ok_or_else(|| anyhow!("Repository has no working directory (bare repo?)"))?
        .to_path_buf();

    let claude_dir = repo_root.join(".claude");
    create_dir_all(&claude_dir)?;
    let settings_path = claude_dir.join("settings.local.json");

    // Read existing settings or create an empty object
    let mut settings = settings_path
        .exists()
        .then(|| read_to_string(&settings_path).ok())
        .flatten()
        .and_then(|content| from_str::<Value>(&content).ok())
        .filter(|v| v.is_object())
        .unwrap_or_else(|| json!({}));
    let settings = settings.as_object_mut().unwrap();

    // Create the new hook entry
    settings
        .entry("hooks".to_string())
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .unwrap()
        .entry("SessionStart".to_string())
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .unwrap()
        .push(json!({
            "hooks": [
                {
                    "type": "command",
                    "command": format!("{} --language {language}", current_exe()?.display()),
                    "timeout": 10
                }
            ]
        }));

    File::create(&settings_path)?.write_all(to_string_pretty(&settings)?.as_bytes())?;

    println!("Hook installed successfully to {}", settings_path.display());

    Ok(())
}
