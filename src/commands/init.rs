
use crate::{config::*, git, ops::write_log, ui, ops};
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

    // Initialize the git repository
    // if the local is false, use repo_url to add remote
    let remote_url = if config.storage.local {
        None
    } else if !config.storage.repo_url.is_empty() {
        Some(config.storage.repo_url.as_str())
    } else {
        None
    };
    match git::init_repo(profile,remote_url) {
        Ok(_) => {
            write_log("info", "INIT", "Git repository initialized successfully", None).unwrap();
          
        },
        Err(e) => {
            write_log("error", "INIT", &format!("Error initializing git repository: {}", e), None).unwrap();
            eprintln!("Error initializing git repository: {}", e);
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