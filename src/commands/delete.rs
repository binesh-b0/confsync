use crate::cli::DeleteTarget;
use crate::config::{check_config_exists, delete_config};
use crate::{git, ui};
use crate::ops::write_log;
use crate::ui::printer;

pub fn handle_delete (target: DeleteTarget, profile: &str) {
    match target {
        DeleteTarget::Config { force } =>{
            if force {
                if let Err(e) = delete_config() {
                    write_log("error", "DELETE", &format!("Error deleting config: {}", e), None).unwrap();
                    printer(format!("Error deleting config: {}", e).as_str(),ui::MessageType::Error);
                } else {
                    write_log("info", "DELETE", "confSync config deleted", None).unwrap();
                    printer("confSync config deleted.😔", ui::MessageType::Info);
                }
            } else if check_config_exists() {
                printer("Use --force to delete.", ui::MessageType::Warning);
            } else {
                printer("No config file found.", ui::MessageType::Info);
                write_log("info", "DELETE", "No config file found", None).unwrap();
            }
        },
        DeleteTarget::Local { force } => {
            if force {
                if let Err(e) = git::delete_repo(true, false, &profile) {
                    write_log("error", "DELETE", &format!("Error deleting local repo: {}", e), None).unwrap();
                    printer(format!("Error deleting local repo: {}", e).as_str(), ui::MessageType::Error);
                } else {
                    write_log("info", "DELETE", "Local repository deleted", None).unwrap();
                    printer("Local repository deleted.😔", ui::MessageType::Info);
                }
            } else {
                printer("Use --force to delete.", ui::MessageType::Warning);
                write_log("warn", "DELETE", "Attempt to delete local repo without force flag", None).unwrap();
            }
        },
        DeleteTarget::Remote { force } => {
            if force {
                if let Err(e) = git::delete_repo(false, true, &profile) {
                    write_log("error", "DELETE", &format!("Error deleting remote repo: {}", e), None).unwrap();
                    printer(format!("Error deleting remote repo: {}", e).as_str(), ui::MessageType::Error);
                } else {
                    write_log("info", "DELETE", "Remote repository deleted", None).unwrap();
                    printer("Remote repository deleted.", ui::MessageType::Info);
                }
            } else {
                printer("Use --force to delete.", ui::MessageType::Warning);
            }
        },
        DeleteTarget::All { force } => {
            if force {
                if let Err(e) = git::delete_repo(true, true, &profile) {
                    write_log("error", "DELETE", &format!("Error deleting everything: {}", e), None).unwrap();
                    printer(format!("Error deleting all repos: {}", e).as_str(), ui::MessageType::Error);
                } else {
                    write_log("info", "DELETE", "Deleted all", None).unwrap();
                    printer("All repositories deleted.", ui::MessageType::Info);
                }
                if let Err(e) = delete_config() {
                    write_log("error", "DELETE", &format!("Error deleting config: {}", e), None).unwrap();
                    printer(format!("Error deleting config: {}", e).as_str(), ui::MessageType::Error);
                } else {
                    write_log("info", "DELETE", "confsync config deleted", None).unwrap();
                    printer("confsync config deleted.", ui::MessageType::Info);
                }
            } else {
                printer("Use --force to delete.", ui::MessageType::Warning);
            }
        }
    }
}