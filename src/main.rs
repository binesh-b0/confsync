
use clap::{CommandFactory, Parser};

mod cli;
mod config;
mod git;
mod ops;
mod commands;
mod ui;

use commands::init::handle_init;
use commands::add::handle_add;

use cli::{Cli, ConfigCommands, DeleteTarget};
use config::{
    check_config_exists, default_config_path, delete_config, load_config, view_config, is_tracked
};
use ops::{copy_file_to_repo, write_log};
use ui::printer;


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
            cli::Commands::Init { repo_url, local, force } => 
                handle_init(repo_url, local, force,None),
            cli::Commands::Add { path,name } => 
                handle_add(path, name, &profile),
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
                    Ok(output) => printer(&output,ui::MessageType::Git),
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
                    if let Err(e) = copy_file_to_repo(path.clone(), alias.as_str(), &profile,false) {
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
                        printer("DOne", ui::MessageType::Default);
                    }
                    
                }
                
            }
            cli::Commands::List { tracked , alias   } => {
                // list the tracked files
                if tracked {
                    if let Err(e) = config::list_tracked_files() {
                        printer(format!("Error listing tracked files: {}", e).as_str(), ui::MessageType::Error);
                    }
                }
                else {
                    // list the history of the file
                    if let Some(alias_value) = alias.as_deref() {
                        // get timestamp from the cmt file
                        match ops::read_cmt(alias_value, &profile) {
                            Ok(lines) => {
                                if lines.is_empty() {
                                    ui::printer("No history found",ui::MessageType::Error);
                                    write_log("info", "LIST", &format!("No history found for {}", alias_value), None).unwrap();
                                } else {
                                    ui::printer(format!("=== {} === ", alias_value).as_str(),ui::MessageType::Info);
                                    for line in lines {
                                        ui::printer(&line,ui::MessageType::Default);
                                    }
                                }
                            }
                            Err(e) => {
                                printer(format!("Error reading cmt file: {}", e).as_str(), ui::MessageType::Error);
                            }
                        }
                    } else {
                        printer("Alias not provided", ui::MessageType::Error);
                    }
                }
                
            }
            cli::Commands::Version => {
                println!("Version: {}", env!("CARGO_PKG_VERSION"));
                write_log("info", "VERSION", "Version command executed", None).unwrap();
            }
            _ => {
                println!("other command");
                write_log("warn", "MAIN", "I have no code for that", None).unwrap();
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