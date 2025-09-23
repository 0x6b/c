use std::path::Path;

use anyhow::{Context, Result};
use git2::{DiffFormat, DiffOptions, Signature, Time};
use jiff::Zoned;

use crate::types::Repository;

/// Stages a single file for the next commit
///
/// # Arguments
/// * `repo` - The git repository
/// * `file_path` - Path to the file to stage
pub fn stage_file(repo: &Repository, file_path: &str) -> Result<()> {
    let mut index = repo.index()?;
    index
        .add_path(Path::new(file_path))
        .with_context(|| format!("Failed to add file to index: {}", file_path))?;
    index.write()?;
    Ok(())
}

/// Stages all modified files in the working directory
///
/// # Arguments
/// * `repo` - The git repository
pub fn stage_all_files(repo: &Repository) -> Result<()> {
    let mut index = repo.index()?;
    index.add_all(["."], git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok(())
}

/// Gets the diff content for currently staged changes
///
/// # Arguments
/// * `repo` - The git repository
///
/// # Returns
/// The diff as a string, truncated to 5000 characters if too long.
/// Returns an error if the diff cannot be generated.
pub fn get_staged_diff(repo: &Repository) -> Result<String> {
    let head = repo.head()?.peel_to_tree()?;
    let index = repo.index()?;
    let mut opts = DiffOptions::new();
    opts.force_text(false);
    let diff = repo.diff_tree_to_index(Some(&head), Some(&index), Some(&mut opts))?;

    let mut diff_text = String::new();
    diff.print(DiffFormat::Patch, |_, _, line| {
        if let Ok(content) = std::str::from_utf8(line.content()) {
            match line.origin() {
                '+' | '-' | ' ' => diff_text.push_str(&format!("{}{content}", line.origin())),
                _ => diff_text.push_str(content),
            }
        }
        true
    })?;

    let diff_text = diff_text.trim();
    Ok(if diff_text.len() > 5000 {
        format!("{}\\n\\n[... truncated ...]", &diff_text[..5000])
    } else {
        diff_text.to_string()
    })
}

/// Creates a git commit with the given message
///
/// # Arguments
/// * `repo` - The git repository
/// * `message` - The commit message
pub fn create_commit(repo: &Repository, message: &str) -> Result<()> {
    let signature = create_signature(repo)?;
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let parents: Vec<_> = repo
        .head()
        .ok()
        .and_then(|head| head.target())
        .and_then(|oid| repo.find_commit(oid).ok())
        .map(|commit| vec![commit])
        .unwrap_or_default();

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents.iter().collect::<Vec<_>>(),
    )?;

    Ok(())
}

/// Creates a git signature from git config with conditionally includes support
///
/// # Arguments
/// * `repo` - The git repository
///
/// # Returns
/// A git signature with user info from the appropriate configuration, or repository default
fn create_signature(repo: &Repository) -> Result<Signature<'_>> {
    // Try to get user config with conditional includes support using gix
    if let Ok((name, email)) = get_git_config(repo) {
        let now = Time::new(Zoned::now().timestamp().as_second(), 0);
        return Ok(Signature::new(&name, &email, &now)?);
    }

    // Fall back to the repository signature if all else fails
    Ok(repo.signature()?)
}

/// Gets user configuration using gix with automatic conditional includes resolution
fn get_git_config(repo: &Repository) -> Result<(String, String)> {
    // Get the repository path for gix
    let repo_path = repo.path().parent().unwrap_or_else(|| repo.path());

    // Open a repository with gix to get config with conditional includes resolved
    let gix_repo = gix::open(repo_path)?;
    let config = gix_repo.config_snapshot();

    // Extract username and email
    let name = config
        .string("user.name")
        .ok_or_else(|| anyhow::anyhow!("user.name not found in config"))
        .and_then(|n| {
            std::str::from_utf8(&n)
                .map(|s| s.to_string())
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in user.name: {}", e))
        })?;

    let email = config
        .string("user.email")
        .ok_or_else(|| anyhow::anyhow!("user.email not found in config"))
        .and_then(|e| {
            std::str::from_utf8(&e)
                .map(|s| s.to_string())
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in user.email: {}", e))
        })?;

    Ok((name, email))
}

/// Gets the current branch name
///
/// # Arguments
/// * `repo` - The git repository
///
/// # Returns
/// The current branch name, or "HEAD" if detached
pub fn get_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head()?;
    if let Some(branch_name) = head.shorthand() {
        Ok(branch_name.to_string())
    } else {
        Ok("HEAD".to_string())
    }
}

/// Creates a new session branch with timestamp
///
/// # Arguments
/// * `repo` - The git repository
/// * `session_id` - The session identifier
///
/// # Returns
/// `Ok(())` on success, or an error if the branch cannot be created. The branch name follows the
/// format: `session/{session_id}_{timestamp}`
pub fn create_session_branch(repo: &Repository, session_id: &str) -> Result<()> {
    let timestamp = Zoned::now().strftime("%Y%m%d_%H%M%S");
    let branch_name = format!("session/{}_{}", session_id, timestamp);
    let head_commit = repo.head()?.peel_to_commit()?;

    repo.branch(&branch_name, &head_commit, false)?;
    repo.set_head(&format!("refs/heads/{}", branch_name))?;
    repo.checkout_head(None)?;

    Ok(())
}
