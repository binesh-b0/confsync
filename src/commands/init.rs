use crate::{config::*, repo, ops::write_log, ui, ops};
/// Initializes the repository configuration and sets up the repository.
///
/// This function sets up the configuration for a repository, optionally using a provided repository URL, force reinitialization flag, and profile selection. If a configuration already exists and `force` is not set, initialization is aborted. The function saves the configuration, initializes the repository, and copies the configuration file into the repository. If `git` is false, the operation is not performed.
///
/// # Parameters
/// - `repo_url`: Optional URL of the repository to initialize. If not provided or empty, a local repository is assumed.
/// - `git`: If false, initialization is not performed and an error is reported.
/// - `force`: If true, forces reinitialization even if a configuration already exists.
/// - `profile`: Optional profile name to use for initialization; defaults to "default" if not provided.
///
/// # Examples
///
/// ```
/// handle_init(Some("https://example.com/repo.git".to_string()), true, false, Some("work".to_string()));
/// ```
pub fn handle_init(repo_url: Option<String>, git: bool, force: bool, profile: Option<String>) {
    // if not git, print not yet implemented
    if !git {
        ui::printer("Not yet implemented", ui::MessageType::Error);
        write_log("error", "INIT", "Git support not yet implemented", None).unwrap();
        return;
    }
    //  if repo_url is None or empty, set local to true
    let local = if let Some(url) = repo_url.as_ref() {
        !url.is_empty()
    } else {
        true
    };
    let profile = profile.as_deref().unwrap_or("default");
    // load or create config
    let mut config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            write_log("error", "INIT", &format!("Error loading config: {}", e), None).unwrap();
            println!("Error loading config: {}", e);
            Config::default()
        }
    };
    // Prevent overwriting existing config if not forced  
    if check_config_exists() && !force {
        ui::printer("Already up and running", ui::MessageType::Success);
        ui::printer("\nuse --force to reinitalize", ui::MessageType::Default);
        write_log("info", "INIT", "Init aborted: config already exists", None).unwrap();
        return;
    }
    // Update config with new repo URL and storage option
    config.storage.repo_url = repo_url.unwrap_or_default();
    config.storage.local = local;
    
    // Save the updated config
    if let Err(e) = save_config(&config) {
        write_log("error", "INIT", &format!("Error saving config: {}", e), None).unwrap();
        eprintln!("Error saving config: {}", e);
        return;
    } else {
        write_log("info", "INIT", "Config saved successfully", None).unwrap();
    }

    // Initialize repository
    match repo::init_repo(profile) {
        Ok(_) => {
            write_log("info", "INIT", "Repository initialized successfully", None).unwrap();
        },
        Err(e) => {
            write_log("error", "INIT", &format!("Error initializing repository: {}", e), None).unwrap();
            eprintln!("Error initializing repository: {}", e);
            std::process::exit(1);
        }
    }

    // get the path of the config file
    let config_path = default_config_path().unwrap();

    // copy the config file to the repo
    ops::copy_file_to_repo(config_path,"confsync", "default",true).unwrap_or_else(|e| {
        write_log("error", "INIT", &format!("Error copying config file to repo: {}", e), None).unwrap();
        eprintln!("Error copying config file to repo: {}", e);
    });
    
    ui::printer("✅ init completed", ui::MessageType::Success);
    ui::printer("use `confsync add` to add files", ui::MessageType::Default);
    
}
