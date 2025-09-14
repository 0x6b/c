use std::{
    env::var,
    io::{Read, stdin},
};

use clap::Parser;
use daemonize::Daemonize;
use serde_json::from_str;

mod commit_message_generator;
mod committer;
mod git_ops;
mod types;

use commit_message_generator::CommitMessageGenerator;
use committer::Committer;

use crate::types::HookEvent;

#[derive(Parser)]
#[clap(version, about)]
pub struct Args {
    /// Language to use for commit messages
    #[arg(short, long, default_value = "Japanese", env = "CC_AUTO_COMMIT_LANGUAGE")]
    pub language: String,
}

fn main() -> anyhow::Result<()> {
    // Prevent recursive calls
    if var("CLAUDE_AUTO_COMMIT_RUNNING").is_ok() {
        return Ok(());
    }

    let Args { language } = Args::parse();

    let mut input = String::new();
    stdin().read_to_string(&mut input)?;

    match from_str::<HookEvent>(&input) {
        Ok(hook_event) => {
            match Daemonize::new()
                .working_directory(hook_event.cwd())
                .umask(0o027)
                .start()
            {
                Ok(_) => Committer::new().handle_event(hook_event, language),
                Err(e) => {
                    eprintln!("Error starting daemon: {e}");
                    Err(e.into())
                }
            }
        }
        Err(_) => {
            println!("{}", CommitMessageGenerator::new(language)?.generate(&input));
            Ok(())
        }
    }
}
