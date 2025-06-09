use chrono::Local;
use directories::ProjectDirs;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::ops::write_log;

/// Initialize a new repository directory for the given profile.
/// Simply creates the directory if it does not exist.
pub fn init_repo(profile: &str) -> Result<PathBuf, String> {
    let project_dirs = ProjectDirs::from("", "", "confsync")
        .ok_or_else(|| "Failed to find config directory".to_string())?;
    let repo_path = project_dirs.data_dir().join(profile);
    fs::create_dir_all(&repo_path)
        .map_err(|e| format!("Failed to create repository: {}", e))?;
    write_log("info", "INIT", "Initialized repository", Some(profile.to_string()))?;
    Ok(repo_path)
}

/// Record a commit message. A simple history.log file is used.
pub fn commit(profile: &str, message: &str) -> Result<(), String> {
    let project_dirs = ProjectDirs::from("", "", "confsync")
        .ok_or_else(|| "Failed to find config directory".to_string())?;
    let repo_path = project_dirs.data_dir().join(profile);
    if !repo_path.exists() {
        return Err("Repository does not exist".into());
    }
    let log_file = repo_path.join("history.log");
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_file)
        .map_err(|e| format!("Failed to open history log: {}", e))?;
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(file, "[{}] {}", timestamp, message)
        .map_err(|e| format!("Failed to write history: {}", e))?;
    write_log(
        "info",
        "COMMIT",
        &format!("Message recorded: {}", message),
        Some(profile.to_string()),
    )?;
    Ok(())
}

/// Remove the repository directory for the profile.
pub fn delete_repo(profile: &str) -> Result<(), String> {
    let project_dirs = ProjectDirs::from("", "", "confsync")
        .ok_or_else(|| "Failed to find config directory".to_string())?;
    let repo_path = project_dirs.data_dir().join(profile);
    if repo_path.exists() {
        fs::remove_dir_all(&repo_path)
            .map_err(|e| format!("Failed to delete repository: {}", e))?;
    }
    write_log("info", "DELETE", "Repository deleted", Some(profile.to_string()))?;
    Ok(())
}

/// Read commit messages from history log.
pub fn list_history(profile: &str) -> Result<Vec<String>, String> {
    let project_dirs = ProjectDirs::from("", "", "confsync")
        .ok_or_else(|| "Failed to find config directory".to_string())?;
    let repo_path = project_dirs.data_dir().join(profile);
    let log_file = repo_path.join("history.log");
    if !log_file.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&log_file)
        .map_err(|e| format!("Failed to read history log: {}", e))?;
    Ok(content.lines().map(|s| s.to_string()).collect())
}
