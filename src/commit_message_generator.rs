use std::{process::Command, sync::LazyLock};

use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use toml::from_str;

#[derive(Deserialize)]
struct Config {
    prompt: Prompt,
    generator: Generator,
}

#[derive(Deserialize)]
struct Prompt {
    template: String,
}

#[derive(Deserialize)]
struct Generator {
    command: String,
    args: Vec<String>,
    default_commit_message: String,
}

static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    from_str(include_str!("../assets/commit-config.toml"))
        .expect("Failed to parse embedded commit-config.toml")
});

static CONVENTIONAL_COMMIT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z]+:\s.+").expect("Failed to compile conventional commit regex")
});

/// Generates commit messages using AI based on git diff content
#[derive(Default)]
pub struct CommitMessageGenerator {
    prompt_template: &'static str,
    command: &'static str,
    args: &'static [String],
    language: &'static str,
}

impl CommitMessageGenerator {
    /// Creates a new commit message generator for the specified language
    ///
    /// # Arguments
    /// - `language` - The language to use for generating commit messages
    pub fn new(language: &str) -> Result<Self> {
        Ok(Self {
            prompt_template: &CONFIG.prompt.template,
            command: &CONFIG.generator.command,
            args: &CONFIG.generator.args,
            language: Box::leak(Box::new(language.to_string())),
        })
    }

    /// Generates a commit message from the provided diff content
    ///
    /// # Arguments
    /// - `diff_content` - The git diff content to analyze for message generation
    ///
    /// # Returns
    /// A generated commit message string. If generation fails or the result doesn't follow a
    /// conventional commit format, returns a default commit message.
    pub fn generate(&self, diff_content: &str) -> String {
        self.try_generate(diff_content)
            .map(|message| {
                if CONVENTIONAL_COMMIT_RE.is_match(message.lines().next().unwrap_or("").trim()) {
                    message
                } else {
                    format!("{}\n\n{message}", CONFIG.generator.default_commit_message)
                }
            })
            .unwrap_or_else(|| CONFIG.generator.default_commit_message.to_string())
    }

    fn try_generate(&self, diff_content: &str) -> Option<String> {
        let prompt = self
            .prompt_template
            .replace("{language}", self.language)
            .replace("{diff_content}", diff_content);

        Command::new(self.command)
            .env("CLAUDE_AUTO_COMMIT_RUNNING", "1") // To prevent recursive calls
            .args(self.args.iter())
            .arg(&prompt)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .filter(|message| !message.is_empty())
    }
}
