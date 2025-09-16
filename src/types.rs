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
        #[allow(dead_code)] // for future use
        transcript_path: String,
        cwd: String,
        #[serde(default)]
        source: Option<SessionStartSource>,
    },
    PostToolUse {
        #[allow(dead_code)] // for future use
        session_id: String,
        #[allow(dead_code)] // for future use
        transcript_path: String,
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

    /// Gets the session ID from the hook event
    ///
    /// # Returns
    /// The session ID as a string slice
    #[allow(dead_code)] // for future use
    pub fn session_id(&self) -> &str {
        match self {
            HookEvent::SessionStart { session_id, .. }
            | HookEvent::PostToolUse { session_id, .. } => session_id,
        }
    }

    /// Gets the transcript path from the hook event
    ///
    /// # Returns
    /// The transcript file path as a string slice
    #[allow(dead_code)] // for future use
    pub fn transcript_path(&self) -> &str {
        match self {
            HookEvent::SessionStart { transcript_path, .. }
            | HookEvent::PostToolUse { transcript_path, .. } => transcript_path,
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
#[serde(rename_all = "snake_case")]
#[allow(dead_code)] // for future use
pub enum SessionEndReason {
    Clear,
    Logout,
    PromptInputExit,
    Exit,
    Other,
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
}
