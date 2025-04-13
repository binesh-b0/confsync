
use chrono::Utc;
use directories::ProjectDirs;
use std::{fs, path::Path};
use std::process::Command;
use git2::{DiffOptions, Repository};

use crate::ops::write_log;


/// Initialize a new git repository
pub fn init_repo(profile: &str, repo_url: Option<&str>) -> Result<(), String> {
    let project_dirs = ProjectDirs::from("", "", "confsync")
        .ok_or("Failed to find config directory")?;
    
    let repo_path = project_dirs.data_dir().join(profile);
    
    // Create parent directories if needed
    fs::create_dir_all(repo_path.clone())
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Initialize repo with default branch "main"
    let mut opts = git2::RepositoryInitOptions::new();
    opts.initial_head("main");
    let repo = Repository::init_opts(repo_path.clone(), &opts)
        .map_err(|e| format!("Repo init failed: {}", e))?;

    
    // Set to prefer rebase on pull
    {
        let mut config = repo.config().map_err(|e| format!("Failed to get config: {}", e))?;
        config.set_bool("pull.rebase", true)
            .map_err(|e| format!("Failed to set pull.rebase configuration: {}", e))?;
    }
    
    // Set remote if provided
    if let Some(url) = repo_url {
        repo.remote("origin", url)
            .map_err(|e| format!("Failed to set remote: {}", e))?;
    }
    write_log("info", "INIT", "Initialized repo", None).unwrap();
    // println!("Initialized repository at: {}", repo_path.display());
    Ok(())
}

/// Forward the git commands to the git CLI
pub fn git_command(args: &[&str]) -> Result<String,String> {
    // Check if git is installed
    if !is_git_installed() {
        return Err("Git is not installed".into());
    }
    // Check if the command is valid
    if args.is_empty() {
        return Err("No git command provided".into());
    }
    // the git commands should be excecuted in the project directory
    let project_dirs = ProjectDirs::from("","","confsync")
        .ok_or("Failed to get project directories")?;
    let repo_path = project_dirs.data_dir().join("default");
    if !repo_path.exists() {
        return Err("Repository does not exist".into());
    }
    // Execute the git command
    let output = Command::new("git")
        .args(args)
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("Failed to execute git command: {}", e))?;
    // Check if the command was successful
    if !output.status.success() {
        return Err(format!(
            "Git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    write_log("info", "GIT", &format!("Git command: {:?}", args), None).unwrap();
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}


/// Check if git is installed
pub fn is_git_installed() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Commit and push 
pub fn commit_and_push(profile: &str, message: &str, push: bool) -> Result<(), String> {
    let project_dirs = ProjectDirs::from("", "", "confsync")
        .ok_or("Failed to find config directory")?;
    
    let repo_path = project_dirs.data_dir().join(profile);
    // Check if the repository exists
    if !repo_path.exists() {
        return Err("Repository does not exist".into());
    }

    // add all changes
    let _ = git_command(&["add", "."])?;
    
    // Commit changes
    let output = git_command(&["commit", "-m", message])?;
    write_log("info", "COMMIT", &format!("Commit output: {}", output), Some(profile.to_string()))?;
    

    // Push changes if requested
    if push {
        // first pull to ensure we are up to date
        let output = git_command(&["pull"])?;
        write_log("info", "PULL", &format!("Pull output: {}", output), Some(profile.to_string()))?;

        let output = git_command(&["push"])?;
        write_log("info", "PUSH", &format!("Push output: {}", output), Some(profile.to_string()))?;
    }
    
    Ok(())
}

/// Delete the local and/or remote repository
pub fn delete_repo(local: bool, remote: bool,profile: &str) -> Result<(), String> {
    let project_dirs = ProjectDirs::from("", "", "confsync")
    .ok_or("Failed to find config directory")?;

    let repo_path = project_dirs.data_dir().join(profile);

    // Check if the repository exists
    if !repo_path.exists() {
        return Err("Repository does not exist".into());
    }
    if local {
        // Delete the local repository
        fs::remove_dir_all(&repo_path)
            .map_err(|e| format!("Failed to delete: {}", e))?;
        println!("Local repository deleted: {}", repo_path.display());
    }
    if remote {
        // Delete the remote repository
        let output = git_command(&["push", "--delete", "origin", "main"])?;
        println!("Remote branch deleted: {}", output);
    }

    Ok(())
}

/// Get a file's commit history using git2
/// return the datetime of the commits in a list of strings
pub fn _get_commit_history(alias: &str, profile: &str) -> Result<Vec<String>, String> { 
    // Locate the configuration directory.
    let project_dirs = ProjectDirs::from("", "", "confsync")
        .ok_or("Failed to find config directory")?;
    
    // Use the provided profile ("default" by default) to build the repo path.
    let repo_path = project_dirs.data_dir().join(profile);
    
    // Verify that the repository exists.
    if !repo_path.exists() {
        return Err("Repository does not exist".into());
    }
    
    // Open the repository.
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    
    // The relative path to the specific file is: alias/file.
    let target_file = Path::new(alias).join("file");

    // Create a revwalk to iterate over commit history.
    let mut revwalk = repo.revwalk()
        .map_err(|e| format!("Failed to create revwalk: {}", e))?;
    revwalk.push_head()
        .map_err(|e| format!("Failed to push head: {}", e))?;

    // This vector will hold the formatted commit timestamps.
    let mut commit_dates = Vec::new();

    // Iterate over commits.
    for oid_result in revwalk {
        let oid = oid_result.map_err(|e| format!("Failed to get oid: {}", e))?;
        let commit = repo.find_commit(oid)
            .map_err(|e| format!("Failed to find commit: {}", e))?;

        // A flag to mark if the target file was changed.
        let mut file_changed = false;
        
        // If this commit has no parent (i.e. it's the initial commit)
        if commit.parent_count() == 0 {
            let tree = commit.tree()
                .map_err(|e| format!("Failed to get tree: {}", e))?;
            // Check if the target file exists in the initial commit.
            if tree.get_path(&target_file).is_ok() {
                file_changed = true;
            }
        } else {
            // Compare against the first parent.
            let parent = commit.parent(0)
                .map_err(|e| format!("Failed to get parent commit: {}", e))?;
            let parent_tree = parent.tree()
                .map_err(|e| format!("Failed to get parent tree: {}", e))?;
            let curr_tree = commit.tree()
                .map_err(|e| format!("Failed to get commit tree: {}", e))?;

            // Initialize diff options.
            let mut diff_opts = DiffOptions::new();
            // Create the diff between the first parent and the current commit.
            let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&curr_tree), Some(&mut diff_opts))
                .map_err(|e| format!("Failed to generate diff: {}", e))?;
            
            // Iterate through diff entries.
            diff.foreach(
                // For each delta, check if the target file path is present.
                &mut |delta, _| {
                    if let Some(new_path) = delta.new_file().path() {
                        if new_path == target_file.as_path() {
                            file_changed = true;
                        }
                    }
                    if let Some(old_path) = delta.old_file().path() {
                        if old_path == target_file.as_path() {
                            file_changed = true;
                        }
                    }
                    true
                },
                None,
                None,
                None
            ).map_err(|e| format!("Failed to iterate over diff: {}", e))?;
        }
        
        // If the commit touched the target file, record its timestamp.
        if file_changed {
            let dt = chrono::TimeZone::timestamp_opt(&Utc, commit.time().seconds(), 0)
                .single()
                .ok_or("Invalid timestamp")?;
            commit_dates.push(dt.format("%Y-%m-%d %H:%M:%S").to_string());
        }
    }
    
    Ok(commit_dates)
}