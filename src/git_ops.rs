use std::path::Path;

use anyhow::{Context, Result};
use git2::{DiffFormat, DiffOptions};
use jiff::Zoned;

use crate::types::Repository;

/// Stage a single file
pub fn stage_file(repo: &Repository, file_path: &str) -> Result<()> {
    let mut index = repo.index()?;
    index
        .add_path(Path::new(file_path))
        .with_context(|| format!("Failed to add file to index: {}", file_path))?;
    index.write()?;
    Ok(())
}

/// Stage all modified files in the working directory
pub fn stage_all_changes(repo: &Repository) -> Result<()> {
    let mut index = repo.index()?;
    index.add_all(["."], git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok(())
}

/// Get diff content for currently staged changes
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
                '+' | '-' | ' ' => diff_text.push_str(&format!("{}{}", line.origin(), content)),
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

/// Create a git commit with the given message
pub fn create_commit(repo: &Repository, message: &str) -> Result<()> {
    let signature = repo.signature()?;
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

/// Get the current branch name
pub fn get_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head()?;
    if let Some(branch_name) = head.shorthand() {
        Ok(branch_name.to_string())
    } else {
        Ok("HEAD".to_string())
    }
}

/// Create a new session branch
pub fn create_session_branch(repo: &Repository, session_id: &str) -> Result<()> {
    let timestamp = Zoned::now().strftime("%Y%m%d_%H%M%S");
    let branch_name = format!("session/{}_{}", session_id, timestamp);
    let head_commit = repo.head()?.peel_to_commit()?;

    repo.branch(&branch_name, &head_commit, false)?;
    repo.set_head(&format!("refs/heads/{}", branch_name))?;
    repo.checkout_head(None)?;

    Ok(())
}
