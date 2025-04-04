use std::path::PathBuf;

use clap::Parser;

mod cli;
mod config;
mod git;
mod ops;

use cli::{Cli, ConfigCommands, DeleteTarget};
use config::{
    check_config_exists,
    default_config_path,
    delete_config, 
    load_config, 
    save_config,
    view_config, 
    add_tracking_file,
    Config
};
use ops::copy_file_to_repo;


fn main(){
    let cli = Cli::parse();
    match cli.command {
        cli::Commands::Init {repo_url, local, force} => {
            // load or create config
            let mut config = match load_config() {
                Ok(config) => config,
                Err(e) => {
                    println!("Error loading config: {}", e);
                    Config::default()
                }
            };
            // Prevent overwriting existing config if not forced  
            if check_config_exists() && !force {
                println!("Config already exists. Use --force to reinitialize.");
                return;
            }
            // Update config with new repo URL and storage option
            config.storage.repo_url = repo_url.unwrap_or_default();
            config.storage.local = local;
            
            // Save the updated config
            if let Err(e) = save_config(&config) {
                eprintln!("Error saving config: {}", e);
                return;
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
            match git::init_repo(remote_url) {
                Ok(_) => {
                    println!("Git initialized");
                    println!("Welcome to confSync! \n\
                    Your configuration files will be stored at: \n\
                    {} \n\
                    Add files to be tracked using the `add` command.",
                    default_config_path().unwrap().display());
                },
                Err(e) => {
                    eprintln!("Error initializing git repository: {}", e);
                    std::process::exit(1);
                }
            }
        },
        cli::Commands::Add { path,name } => {
            // check if config file exists
            if !check_config_exists() {
                println!(" Please run `confsync init` to initialize.");
                return;
            }
            // path to PathBuf
            let path = match PathBuf::from(path).canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error resolving path: {}", e);
                    return;
                }
            };

            // add to tracking
            match add_tracking_file(path.clone(),name.clone()) {
                Ok(()) => {
                    println!("Added {} to tracking as {}", path.display(), name);
                    // copy the file to the repo
                    if let Err(e) = copy_file_to_repo(path, name.as_str(),"default") {
                        eprintln!("Error copying file to repo: {}", e);
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("Error adding tracking: {}", e);
                    return;
                }
            }
            
        },
        cli::Commands::Delete { target } => {
            match target {
                DeleteTarget::Config { force } =>{
                    if force {
                        if let Err(e) = delete_config() {
                            eprintln!("Error deleting config: {}", e);
                        } else {
                            println!("confSync config deleted.😔");
                        }
                    } else if check_config_exists() {
                        println!(" Use --force to delete.");
                    } else {
                        println!("No config file found.");
                    }
                },
                DeleteTarget::Local { force } => {
                    if force {
                        if let Err(e) = git::delete_repo(true, false) {
                            eprintln!("Error deleting local repo: {}", e);
                        } else {
                            println!("Local repository deleted.😔");
                        }
                    } else {
                        println!("Use --force to delete.");
                    }
                },
                DeleteTarget::Remote { force } => {
                    if force {
                        if let Err(e) = git::delete_repo(false, true) {
                            eprintln!("Error deleting remote repo: {}", e);
                        } else {
                            println!("Remote repository deleted.");
                        }
                    } else {
                        println!("Use --force to delete.");
                    }
                },
                DeleteTarget::All { force } => {
                    if force {
                        if let Err(e) = git::delete_repo(true, true) {
                            eprintln!("Error deleting all repos: {}", e);
                        } else {
                            println!("All repositories deleted.");
                        }
                        if let Err(e) = delete_config() {
                            eprintln!("Error deleting config: {}", e);
                        } else {
                            println!("confsync config deleted.");
                            
                        }
                    } else {
                        println!("Use --force to delete.");
                    }
                }

            }
           
           
        },
        cli::Commands::Config { command } => {
            match command {
                ConfigCommands::Show => {
                    match view_config(false) {
                        Ok(config) => {
                            println!("Config: {:#?}", config);
                        }
                        Err(e) => {
                            eprintln!("Error viewing config: {}", e);
                        }
                    }
                },
                // pipe the config into a text editor
                ConfigCommands::Edit => {
                    match view_config(true) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error editing config: {}", e);
                        }
                    }
                },
            }
        }
        cli::Commands::Git { args } => {
            // Forward the git command to the git CLI
            match git::git_command(&args.iter().map(String::as_str).collect::<Vec<&str>>()) {
                Ok(output) => println!("{}", output),
                Err(e) => eprintln!("Error executing git command: {}", e),
            }
        }
        _ => {
            println!("other command");
        }
    }
}