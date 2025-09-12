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

/// Get diff content for currently staged changes
pub fn get_staged_diff(repo: &Repository) -> Result<String> {
    let head = repo.head()?.peel_to_tree()?;
    let index = repo.index()?;
    let mut diff_opts = DiffOptions::new();
    diff_opts.force_text(false);

    let diff = repo.diff_tree_to_index(Some(&head), Some(&index), Some(&mut diff_opts))?;

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

    Ok(if diff_text.len() > 5000 {
        format!("{}\\n\\n[... truncated ...]", &diff_text[..5000])
    } else {
        diff_text
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
        .and_then(|head| head.peel_to_commit().ok())
        .map(|commit| vec![commit])
        .unwrap_or_default();
    let parent_refs: Vec<_> = parents.iter().collect();

    let commit_id =
        repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &parent_refs)?;
    let commit = repo.find_commit(commit_id)?;

    println!("Commit successful: {message}");
    println!(
        "Commit info: {} {}",
        &commit.id().to_string()[..7],
        commit.message().unwrap_or("").lines().next().unwrap_or("")
    );

    Ok(())
}

/// Create a session branch and switch to it
pub fn create_session_branch(repo: &Repository, session_id: &str) -> Result<String> {
    let timestamp = Zoned::now().strftime("%Y-%m-%d_%H-%M-%S");
    let session_branch = format!("claude-session-{timestamp}-{session_id}");

    let head = repo.head()?;
    let target = head.target().context("No target for HEAD")?;
    let commit = repo.find_commit(target)?;

    repo.branch(&session_branch, &commit, false)?;
    repo.set_head(&format!("refs/heads/{session_branch}"))?;
    repo.checkout_head(None)?;

    println!("Session started: Created branch '{session_branch}'");
    Ok(session_branch)
}

/// Get the current branch name
pub fn get_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head()?;
    Ok(head.shorthand().unwrap_or("HEAD").to_string())
}
