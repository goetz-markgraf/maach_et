use std::process::Command;

/// Checks if the current directory is a git repository and if it has uncommitted changes
///
/// Returns:
/// - None: Not a git repository
/// - Some((bool, Vec<String>)): Git repository with:
///   - bool: whether there are uncommitted changes
///   - Vec<String>: list of uncommitted files
pub fn has_uncommitted_changes() -> Option<(bool, Vec<String>)> {
    // Check if we're in a git repository
    let git_check = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .ok()?;

    if !git_check.status.success() {
        return None;
    }

    // Check for uncommitted changes
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()?;

    if !status.status.success() {
        return None;
    }

    let output = String::from_utf8_lossy(&status.stdout);
    let files: Vec<String> = output.lines().map(|line| line[3..].to_string()).collect();

    // If there's any output, there are uncommitted changes
    Some((!files.is_empty(), files))
}

/// Commits all changes in the repository with the given commit message
///
/// # Arguments
/// * `message` - The commit message to use
///
/// # Returns
/// * `Ok(())` if the commit was successful
/// * `Err(String)` if:
///   - Not in a git repository
///   - No changes to commit
///   - Git commands fail
pub fn commit_all_changes(message: &str) -> Result<(), String> {
    // First check if we're in a git repository
    if has_uncommitted_changes().is_none() {
        return Err("Not in a git repository".to_string());
    }

    // Stage all changes
    let stage = Command::new("git")
        .args(["add", "--all"])
        .output()
        .map_err(|e| format!("Failed to stage changes: {}", e))?;

    if !stage.status.success() {
        return Err(String::from_utf8_lossy(&stage.stderr).to_string());
    }

    // Create the commit
    let commit = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .map_err(|e| format!("Failed to commit: {}", e))?;

    if !commit.status.success() {
        return Err(String::from_utf8_lossy(&commit.stderr).to_string());
    }

    Ok(())
}
