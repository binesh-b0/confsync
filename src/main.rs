
use clap::{CommandFactory, Parser};

mod cli;
mod config;
mod git;
mod ops;
mod commands;
mod ui;

use commands::{delete::handle_delete, init::handle_init};
use commands::add::handle_add;

use cli::{Cli, ConfigCommands};
use config::{
    check_config_exists, default_config_path, load_config, view_config, is_tracked
};
use ops::{copy_file_to_repo, restore_file, write_log};
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
    if cli.version {
        println!("confsync: {}", env!("CARGO_PKG_VERSION"));
        write_log("info", "VERSION", "Version command executed", None).unwrap();
        return; 
    }

    match cli.command {
        Some(command) => match command {
            cli::Commands::Init { remote,git, force } => 
                handle_init(remote, git,force,None),
            cli::Commands::Add { path,name } => 
                handle_add(path, name, &profile),
            cli::Commands::Delete { target } => 
                handle_delete(target, &profile),
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
            cli::Commands::Backup { alias, message, push, force:_ , env } => {
                if !check_config_exists() {
                    println!(" Please run `confsync init` to initialize.");
                    write_log("warn", "BACKUP", "Attempt to backup without config", None).unwrap();
                    return;
                }
                //if env is true, save env variables into a new file in repo
                if env {
                    if !check_config_exists() {
                        println!(" Please run `confsync init` to initialize.");
                        write_log("warn", "BACKUP", "Attempt to backup without config", None).unwrap();
                        return;
                    }
                    if let Err(e) = ops::save_env_vars(&profile) {
                        write_log("error", "BACKUP", &format!("Error saving env vars: {}", e), None).unwrap();
                        eprintln!("Error saving env vars: {}", e);
                        return;
                    } else {
                        ui::printer("Env saved successfully", ui::MessageType::Success);
                        write_log("info", "BACKUP", "Env vars saved successfully", None).unwrap();
                    }
                }
                // if alias is not empty, check its existance
                let alias = alias.unwrap_or_default();
                if !alias.is_empty() {
            
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
            cli::Commands::Restore { target, dry_run: _, overwrite } => {
                // check if file is tracked
                if !is_tracked(target.as_str()) {
                    println!("{} not found", target);
                    write_log("warn", "RESTORE", &format!("{} not found.", target), None).unwrap();
                    return;
                }
                // get the path of the file from alias => dest
                let path= match config::get_path_from_alias(&target) {
                    Ok(path) => path,
                    Err(e) => {
                        write_log("error", "RESTORE", &format!("Error getting path from alias: {}", e), None).unwrap();
                        eprintln!("Error getting path from alias: {}", e);
                        return;
                    }
                };
                // copy the file from the repo to the dest
                if let Err(e) = restore_file(path.clone(), target.as_str(), &profile, overwrite) {
                    write_log("error", "RESTORE", &format!("Error copying file to repo: {}", e), None).unwrap();
                    eprintln!("Error copying file to repo: {}", e);
                    return;
                } else {
                    write_log("info", "RESTORE", &format!("File {} copied from repo successfully", target), None).unwrap();
                    printer("Done", ui::MessageType::Default);  

                }
            },
            cli::Commands::List { alias   } => {
                // list the tracked files if alias is empty
                if alias.is_none() {
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
                
            },
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