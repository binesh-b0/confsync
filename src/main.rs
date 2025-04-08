use std::path::PathBuf;

use clap::{CommandFactory, Parser};

mod cli;
mod config;
mod git;
mod ops;

use cli::{Cli, ConfigCommands, DeleteTarget};
use config::{
    add_tracking_file, check_config_exists, default_config_path, delete_config, load_config, save_config, view_config, Config, is_tracked
};
use ops::{copy_file_to_repo, write_log};


fn main() {
    let cli = Cli::parse();
    // set default profile to "default"
    let profile = cli.profile.unwrap_or_else(|| "default".to_string());

    // Check if the paths argument is set
    if cli.paths {
        let project_dirs = directories::ProjectDirs::from("", "", "confsync").unwrap();
        println!("Config dir: {}", project_dirs.config_dir().display());
        println!("Data dir: {}", project_dirs.data_dir().display());
        println!("Cache dir: {}", project_dirs.cache_dir().display());
        let config_path = default_config_path().unwrap();
        println!("Config path: {}", config_path.display());
        return; // Exit after printing paths
    }

    match cli.command {
        Some(command) => match command {
            cli::Commands::Init { repo_url, local, force } => {
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
                match git::init_repo(&profile,remote_url) {
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
            },
            cli::Commands::Add { path,name } => {
                // check if config file exists
                if !check_config_exists() {
                    println!(" Please run `confsync init` to initialize.");
                    write_log("warn", "ADD", "Attempt to add tracking file without config", None).unwrap();
                    return;
                }
                // path to PathBuf
                let path = match PathBuf::from(path).canonicalize() {
                    Ok(p) => p,
                    Err(e) => {
                        write_log("error", "ADD", &format!("Error resolving path: {}", e), None).unwrap();
                        eprintln!("Error resolving path: {}", e);
                        return;
                    }
                };

                // add to tracking
                match add_tracking_file(path.clone(),name.clone()) {
                    Ok(()) => {
                        write_log("info", "ADD", &format!("Added {} to tracking as {}", path.display(), name), None).unwrap();
                        println!("Added {} to tracking as {}", path.display(), name);
                        // copy the file to the repo
                        if let Err(e) = copy_file_to_repo(path.clone(), name.as_str(), &profile) {
                            write_log("error", "ADD", &format!("Error copying file to repo: {}", e), None).unwrap();
                            eprintln!("Error copying file to repo: {}", e);
                            return;
                        } else {
                            write_log("info", "ADD", &format!("File {} copied to repo successfully", name), None).unwrap();
                        }
                    }
                    Err(e) => {
                        write_log("error", "ADD", &format!("Error adding tracking: {}", e), None).unwrap();
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
                                write_log("error", "DELETE", &format!("Error deleting config: {}", e), None).unwrap();
                                eprintln!("Error deleting config: {}", e);
                            } else {
                                write_log("info", "DELETE", "confSync config deleted", None).unwrap();
                                println!("confSync config deleted.😔");
                            }
                        } else if check_config_exists() {
                            println!(" Use --force to delete.");
                        } else {
                            println!("No config file found.");
                            write_log("info", "DELETE", "No config file found", None).unwrap();
                        }
                    },
                    DeleteTarget::Local { force } => {
                        if force {
                            if let Err(e) = git::delete_repo(true, false, &profile) {
                                write_log("error", "DELETE", &format!("Error deleting local repo: {}", e), None).unwrap();
                                eprintln!("Error deleting local repo: {}", e);
                            } else {
                                write_log("info", "DELETE", "Local repository deleted", None).unwrap();
                                println!("Local repository deleted.😔");
                            }
                        } else {
                            println!("Use --force to delete.");
                            write_log("warn", "DELETE", "Attempt to delete local repo without force flag", None).unwrap();
                        }
                    },
                    DeleteTarget::Remote { force } => {
                        if force {
                            if let Err(e) = git::delete_repo(false, true, &profile) {
                                write_log("error", "DELETE", &format!("Error deleting remote repo: {}", e), None).unwrap();
                                eprintln!("Error deleting remote repo: {}", e);
                            } else {
                                write_log("info", "DELETE", "Remote repository deleted", None).unwrap();
                                println!("Remote repository deleted.");
                            }
                        } else {
                            println!("Use --force to delete.");
                        }
                    },
                    DeleteTarget::All { force } => {
                        if force {
                            if let Err(e) = git::delete_repo(true, true, &profile) {
                                write_log("error", "DELETE", &format!("Error deleting everything: {}", e), None).unwrap();
                                eprintln!("Error deleting all repos: {}", e);
                            } else {
                                write_log("info", "DELETE", "Deleted all", None).unwrap();
                                println!("All repositories deleted.");
                            }
                            if let Err(e) = delete_config() {
                                write_log("error", "DELETE", &format!("Error deleting config: {}", e), None).unwrap();
                                eprintln!("Error deleting config: {}", e);
                            } else {
                                write_log("info", "DELETE", "confsync config deleted", None).unwrap();
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
                                write_log("error", "CONFIG", &format!("Error viewing config: {}", e), None).unwrap();
                                eprintln!("Error viewing config: {}", e);
                            }
                        }
                    },
                    // pipe the config into a text editor
                    ConfigCommands::Edit => {
                        match view_config(true) {
                            Ok(_) => {}
                            Err(e) => {
                                write_log("error", "CONFIG", &format!("Error editing config: {}", e), None).unwrap();
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
            cli::Commands::Backup { alias, message, push, force:_ } => {
                // if alias is not empty, check its existance
                let alias = alias.unwrap_or_default();
                if !alias.is_empty() {
                    if !check_config_exists() {
                        println!(" Please run `confsync init` to initialize.");
                        write_log("warn", "BACKUP", "Attempt to backup without config", None).unwrap();
                        return;
                    }
                    if !is_tracked(alias.as_str()) {
                        println!("{} not found", alias);
                        write_log("warn", "BACKUP", &format!("{} not found.", alias), None).unwrap();
                        return;
                    }
                    // get the path of the file from alias
                    let path = match config::get_path_from_alias(&alias) {
                        Ok(path) => path,
                        Err(e) => {
                            write_log("error", "BACKUP", &format!("Error getting path from alias: {}", e), None).unwrap();
                            eprintln!("Error getting path from alias: {}", e);
                            return;
                        }
                    };
                    // check if the file exists
                    if !path.exists() {
                        println!("File {} not found.", path.display());
                        write_log("warn", "BACKUP", &format!("File {} not found.", path.display()), None).unwrap();
                        return;
                    }
                    // copy the file to the repo
                    if let Err(e) = copy_file_to_repo(path.clone(), alias.as_str(), &profile) {
                        write_log("error", "BACKUP", &format!("Error copying file to repo: {}", e), None).unwrap();
                        eprintln!("Error copying file to repo: {}", e);
                        return;
                    } else {
                        write_log("info", "BACKUP", &format!("File {} copied to repo successfully", alias), None).unwrap();
                    }
                    // commit the changes
                    if let Err(e) = git::commit_and_push(&profile, message.as_deref().unwrap_or(&alias), push && !load_config().unwrap().storage.local) {
                        write_log("error", "BACKUP", &format!("Error committing and pushing: {}", e), None).unwrap();
                        eprintln!("Error committing and pushing: {}", e);
                        return;
                    } else {
                        write_log("info", "BACKUP", "Backup completed successfully", None).unwrap();
                        println!("Backup completed successfully");
                    }
                    
                }
                
            }
            cli::Commands::Version => {
                println!("Version: {}", env!("CARGO_PKG_VERSION"));
                write_log("info", "VERSION", "Version command executed", None).unwrap();
            }
            _ => {
                println!("other command");
                write_log("warn", "MAIN", "Unrecognized command", None).unwrap();
            }
        },
        None => {
            // print all the commands and their descriptions from the clap
            let mut app = Cli::command();
            app.print_help().unwrap();

            // check if the config file exists
            if !check_config_exists() {
                println!("\n\nRun `confsync init` to initialize.");
            }

            
        }
    }
}