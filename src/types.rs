use std::ops::Deref;

use serde::Deserialize;

pub struct Repository {
    inner: git2::Repository,
}

impl Deref for Repository {
    type Target = git2::Repository;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self { inner: git2::Repository::discover(".").unwrap() }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "hook_event_name")]
pub enum HookEvent {
    SessionStart {
        session_id: String,
        cwd: String,
        #[serde(default)]
        source: Option<SessionStartSource>,
    },
    PostToolUse {
        cwd: String,
        tool_name: ToolName,
        tool_input: ToolInput,
        tool_response: ToolResponse,
    },
}

impl HookEvent {
    /// Gets the current working directory from the hook event
    ///
    /// # Returns
    /// The working directory path as a string slice
    pub fn cwd(&self) -> &str {
        match self {
            HookEvent::SessionStart { cwd, .. } | HookEvent::PostToolUse { cwd, .. } => cwd,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ToolInput {
    pub file_path: String,
}

#[derive(Debug, Deserialize)]
pub struct ToolResponse {
    #[serde(default = "default_success")]
    pub success: bool,
}

fn default_success() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStartSource {
    Clear,
    Compact,
    Resume,
    Startup,
    #[serde(other)] // fallback
    Unknown,
}

#[derive(Debug, Deserialize)]
pub enum ToolName {
    Task,
    Bash,
    Glob,
    Grep,
    Read,
    Edit,
    MultiEdit,
    Write,
    WebFetch,
    WebSearch,
    #[serde(other)] // fallback
    Unknown,
}
