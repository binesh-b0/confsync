use chrono::Local;
use directories::ProjectDirs;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::ops::write_log;

/// Initialize a new repository directory for the given profile.
/// Initializes a repository directory for the specified profile, creating it if it does not exist.
///
/// Returns the path to the created or existing repository directory on success, or an error message if initialization fails.
///
/// # Examples
///
/// ```
/// let repo_path = init_repo("default").expect("Failed to initialize repository");
/// assert!(repo_path.ends_with("default"));
/// ```
pub fn init_repo(profile: &str) -> Result<PathBuf, String> {
    let project_dirs = ProjectDirs::from("", "", "confsync")
        .ok_or_else(|| "Failed to find config directory".to_string())?;
    let repo_path = project_dirs.data_dir().join(profile);
    fs::create_dir_all(&repo_path)
        .map_err(|e| format!("Failed to create repository: {}", e))?;
    write_log("info", "INIT", "Initialized repository", Some(profile.to_string()))?;
    Ok(repo_path)
}

/// Appends a timestamped commit message to the history log for the specified profile repository.
///
/// Returns an error if the repository does not exist or if writing to the log fails.
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

/// Deletes the repository directory for the specified profile, removing all its contents.
///
/// Returns an error if the directory cannot be found or deleted. Logs the deletion event.
///
/// # Examples
///
/// ```
/// let result = delete_repo("myprofile");
/// assert!(result.is_ok());
/// ```
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

/// Returns all commit messages from the history log for the specified profile.
///
/// Reads the `history.log` file in the profile's repository directory and returns each commit message as a string in a vector. If the log file does not exist, returns an empty vector.
///
/// # Examples
///
/// ```
/// let history = list_history("default").unwrap();
/// for entry in history {
///     println!("{}", entry);
/// }
/// ```
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
