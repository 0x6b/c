use std::{env::set_current_dir, path::Path};

use anyhow::Result;

use crate::{
    commit_message_generator::CommitMessageGenerator,
    git_ops::{
        create_commit, create_session_branch, get_current_branch, get_staged_diff, stage_all_files,
        stage_file,
    },
    types::{HookEvent, HookEvent::*, Repository, SessionStartSource, ToolName},
};

/// Handles git commit operations for auto-commit functionality
pub struct Committer {
    repo: Repository,
}

impl Committer {
    /// Creates a new Committer instance with a default repository
    pub fn new() -> Self {
        Self { repo: Repository::default() }
    }

    /// Handles different types of hook events and performs appropriate git operations
    ///
    /// # Arguments
    /// * `hook_event` - The hook event to process (SessionStart or PostToolUse)
    /// * `language` - Language to use for generating commit messages
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if any git operation fails
    pub fn handle_event(&self, hook_event: HookEvent, language: &str) -> Result<()> {
        match hook_event {
            SessionStart { session_id, source, cwd, .. } => {
                let current_branch = get_current_branch(&self.repo)?;

                // If the `source` indicates the end of the previous session, commit changes
                if let Some(ref source_value) = source
                    && matches!(
                        source_value,
                        SessionStartSource::Clear
                            | SessionStartSource::Compact
                            | SessionStartSource::Resume
                    )
                {
                    self.handle_session_end(&cwd, &language)?;
                }

                // Then handle new session creation
                if matches!(current_branch.as_str(), "main" | "master" | "develop") {
                    create_session_branch(&self.repo, &session_id)?;
                }
            }
            PostToolUse {
                cwd,
                tool_name: ToolName::Edit | ToolName::MultiEdit | ToolName::Write,
                tool_input,
                tool_response,
                ..
            } if tool_response.success => {
                self.handle_file_commit(&cwd, &tool_input.file_path, language)?;
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_session_end(&self, cwd: &str, language: &str) -> Result<()> {
        set_current_dir(cwd)?;
        stage_all_files(&self.repo)?;
        if !get_staged_diff(&self.repo)?.is_empty() {
            create_commit(
                &self.repo,
                &CommitMessageGenerator::new(language)?.generate(&get_staged_diff(&self.repo)?),
            )?;
        }
        Ok(())
    }

    fn handle_file_commit(&self, cwd: &str, file_path: &str, language: &str) -> Result<()> {
        set_current_dir(cwd)?;

        let relative_path = if Path::new(file_path).is_absolute() {
            Path::new(file_path)
                .strip_prefix(cwd)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| file_path.to_string())
        } else {
            file_path.to_string()
        };

        stage_file(&self.repo, &relative_path)?;
        let diff = get_staged_diff(&self.repo)?;
        if diff.is_empty() {
            return Ok(());
        }

        create_commit(&self.repo, &CommitMessageGenerator::new(language)?.generate(&diff))?;

        Ok(())
    }
}
