use directories::ProjectDirs;
use std::path::Path;
use std::fs;
use std::process::Command;
use git2::{BranchType, Cred, PushOptions, RemoteCallbacks, Repository, Signature};

/// Initialize a new git repository
/// If 'repo_url' is provided, it will set the remote origin,
/// else it will create a local repository (local = true).
pub fn init_repo(repo_url: Option<&str>) -> Result<(), String> {
    // Determine the backup directory
    let project_dirs = ProjectDirs::from("","","confsync")
        .ok_or("Failed to get project directories")?;
    let repo_path = project_dirs.data_dir().join("default");
    println!("Backup directory: {}", repo_path.display());
    // Create the backup directory if it doesn't exist
    if !repo_path.exists() {
        fs::create_dir_all(&repo_path)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    }

    // Check if a git repository already exists
    if repo_path.join(".git").exists() {
        return Err("Repository already exists".into());
    } 
    // Initialize a new repository.
    let repo = Repository::init(&repo_path)
        .map_err(|e| format!("Failed to initialize repository: {}", e))?;
    println!("Initialized empty Git repository in {}", repo_path.display());
    // Write the repository index into a tree.
    let tree_id = {
        let mut index = repo
            .index()
            .map_err(|e| format!("Failed to get repository index: {}", e))?;
        index.write_tree().map_err(|e| format!("Failed to write tree: {}", e))?
    };

    let tree = repo
        .find_tree(tree_id)
        .map_err(|e| format!("Failed to find tree: {}", e))?;

    // Create a signature for the commit.
    let email = match git_command(&["config", "user.email"]) {
        Ok(email) if !email.trim().is_empty() => email.trim().to_owned(),
        _ => "confsync".to_owned(),
    };
    let author = match git_command(&["config", "user.name"]) {
        Ok(name) if !name.trim().is_empty() => name.trim().to_owned(),
        _ => "confsync".to_owned(),
    };
    let sig = Signature::now(&author, &email)
        .map_err(|e| format!("Failed to create signature: {}", e))?;
    println!("Using signature {} <{}>", author, email);

    // Create an initial commit on HEAD (with no parents).
    let commit_id = repo
        .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .map_err(|e| format!("Failed to create initial commit: {}", e))?;

    // Retrieve the commit object.
    let commit = repo
        .find_commit(commit_id)
        .map_err(|e| format!("Failed to find commit: {}", e))?;

    // If branch "main" does not exist, create it.
    if repo.find_branch("main", BranchType::Local).is_err() {
        repo.branch("main", &commit, false)
            .map_err(|e| format!("Failed to create 'main' branch: {}", e))?;
    }
    // Set HEAD to branch "main".
    repo.set_head("refs/heads/main")
        .map_err(|e| format!("Failed to set HEAD to 'main': {}", e))?;

    // If a repository URL is provided, set it as the remote origin and push the initial commit.
    if let Some(url) = repo_url {
        repo.remote("origin", url)
            .map_err(|e| format!("Failed to add remote origin: {}", e))?;
        // Set up callbacks for authentication, if needed.
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _| {
            // Use default credential helper.
            Cred::credential_helper(&repo.config().unwrap(), _url, username_from_url)
        });

        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let mut remote = repo
            .find_remote("origin")
            .map_err(|e| format!("Failed to find remote 'origin': {}", e))?;
        remote
            .push(&["refs/heads/main:refs/heads/main"], Some(&mut push_options))
            .map_err(|e| format!("Failed to push to remote repository: {}", e))?;
    }

    Ok(())
}

/// Forward the git command to the git CLI
/// All the git commands are parsed from the input
/// and the output is returned as a string
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
    let repo_path = project_dirs.data_dir().join("repo");
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

/// Delete the local and/or remote repository
pub fn delete_repo(local: bool, remote: bool) -> Result<(), String> {
    if local {
        // Delete the local repository
        if Path::new(".git").exists() {
            std::fs::remove_dir_all(".git")
                .map_err(|e| format!("Failed to delete local repository: {}", e))?;
        } else {
            return Err("Local repository does not exist".into());
        }
    }

    if remote {
        // Delete the remote repository
        let output = git_command(&["push", "--delete", "origin", "main"])?;
        println!("Remote repository deleted: {}", output);
    }

    Ok(())
}