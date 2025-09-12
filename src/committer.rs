use std::{env::set_current_dir, path::Path};

use anyhow::Result;

use crate::{
    commit_message_generator::CommitMessageGenerator,
    git_ops::{
        create_commit, create_session_branch, get_current_branch, get_staged_diff, stage_file,
    },
    types::{
        HookEvent,
        HookEvent::{PostToolUse, SessionStart},
        Repository, ToolName,
    },
};

pub struct Committer {
    pub repo: Repository,
}

impl Committer {
    pub fn new() -> Self {
        Self { repo: Repository::default() }
    }

    pub fn handle_event(&self, hook_event: HookEvent, language: String) -> Result<()> {
        match hook_event {
            SessionStart { session_id } if get_current_branch(&self.repo)? == "main" => {
                create_session_branch(&self.repo, &session_id)?;
            }
            PostToolUse {
                cwd,
                tool_name: ToolName::Edit | ToolName::MultiEdit | ToolName::Write,
                tool_input,
                tool_response,
            } if tool_response.success => {
                set_current_dir(&cwd)?;

                let file_path = if Path::new(&tool_input.file_path).is_absolute() {
                    Path::new(&tool_input.file_path)
                        .strip_prefix(&cwd)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| tool_input.file_path.clone())
                } else {
                    tool_input.file_path
                };

                stage_file(&self.repo, &file_path)?;

                let diff = get_staged_diff(&self.repo)?;
                if diff.trim().is_empty() {
                    return Ok(());
                }

                create_commit(&self.repo, &CommitMessageGenerator::new(language)?.generate(&diff))?
            }
            _ => {
                return Ok(());
            }
        }

        Ok(())
    }
}
