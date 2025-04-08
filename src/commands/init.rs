use crate::{config::*, git, ops::write_log};

pub fn handle_init(repo_url: Option<String>, local: bool, force: bool, profile: Option<String>) {
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
        println!("Config already exists. Use --force to reinitialize.");
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
            println!("Git initialized");
            println!("Welcome to confSync! \n\
            Your configuration files will be stored at: \n\
            {} \n\
            Add files to be tracked using the `add` command.",
            default_config_path().unwrap().display());
        },
        Err(e) => {
            write_log("error", "INIT", &format!("Error initializing git repository: {}", e), None).unwrap();
            eprintln!("Error initializing git repository: {}", e);
            std::process::exit(1);
        }
    }
}