use std::path::PathBuf;

use crate::ops::write_log;


pub fn handle_add(path: String, name: String, profile: &str) {
       // check if config file exists
       if !crate::config::check_config_exists() {
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
    match crate::config::add_tracking_file(path.clone(),name.clone()) {
        Ok(()) => {
            write_log("info", "ADD", &format!("Added {} to tracking as {}", path.display(), name), None).unwrap();
            println!("Added {} to tracking as {}", path.display(), name);
            // copy the file to the repo
            if let Err(e) = crate::ops::copy_file_to_repo(path.clone(), name.as_str(), &profile,true) {
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
}