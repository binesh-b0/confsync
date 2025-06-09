use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::ui;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub storage: Storage,
    pub tracking: Tracking,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Storage {
    /// stores backup locally or remotely
    pub local: bool,

    /// if not local, stores the remote repo url
    pub repo_url: String,

    /// optional profile name
    pub profile: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tracking {
    #[serde(rename = "files")]
    pub file_map: HashMap<String, PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: Storage {
                local: true,
                repo_url: String::new(),
                profile: Some(String::from("default")),
            },
            tracking: Tracking {
                file_map: HashMap::from_iter([(
                    "confsync".to_string(),
                    default_config_path().unwrap_or_else(|| PathBuf::from("config.toml")),
                )]),
            },
        }
    }
}

/// Get path to the user's config file
pub fn default_config_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "confsync") {
        let conf_dir = proj_dirs.config_dir();
        Some(conf_dir.join("config.toml"))
    } else {
        None
    }
}

/// Check if the config file exists : returns true if it does
pub fn check_config_exists() -> bool {
    default_config_path().map_or(false, |path| path.is_file())
}

/// Load the config file if it exists, or return a default
pub fn load_config() -> Result<Config, String> {
    let path = match default_config_path() {
        Some(p) => p,
        None => return Err("COuld not determine config path".into()),
    };

    if !path.exists() {
        // return default config
        return Ok(Config::default());
    }
    match fs::read_to_string(&path) {
        Ok(contents) => {
            match toml::from_str(&contents) {
                Ok(config) => Ok(config),
                Err(e) => {
                    ui::printer(format!("Warning: Failed to parse config file: {}. Using default configuration.", e).as_str(),ui::MessageType::Error);
                    Ok(Config::default())
                }
            }
        }
        Err(e) => Err(format!("Failed to read config file: {}", e)),
    }
}

/// Write the current config to the default config path,
pub fn save_config(config: &Config) -> Result<(), String> {
    let path = match default_config_path() {
        Some(p) => p,
        None => return Err("Could not determine config path".into()),
    };

    // Ensure the directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let toml_string =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config :{e}"))?;

    fs::write(&path, toml_string).map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// Delete the config file
pub fn delete_config() -> Result<(), String> {
    let path = match default_config_path() {
        Some(p) => p,
        None => return Err("Could not determine config path".into()),
    };

    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete config file: {}", e))?;
    } else {
        return Err("Config file does not exist".into());
    }

    Ok(())
}

/// View/edit the config file or pipe it into a text editor.
pub fn view_config(edit: bool) -> Result<(), String> {
    let path = match default_config_path() {
        Some(p) => p,
        None => return Err("Could not determine config path".into()),
    };

    if !path.exists() {
        return Err("Config file does not exist".into());
    }

    if edit {
        // Open the config file in the nano or the one specified in EDITOR (env var)
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        std::process::Command::new(editor)
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open config file in editor: {}", e))?;
    } else {
        // show the config file in a pager
        use std::process::Stdio;
        let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());
        std::process::Command::new(pager)
            .arg(path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to open config file in pager: {}", e))?;
    }

    Ok(())
}

/// Add a file to the tracking list
pub fn add_tracking_file(path: PathBuf, name: String) -> Result<(), String> {

    let mut config = load_config()?;
    
    // get absolute path
    let abs_path: PathBuf =
        fs::canonicalize(&path).map_err(|e| format!("Failed to get absolute path: {}", e))?;
    if !abs_path.exists() {
        return Err("File does not exist".into());
    }
    if !abs_path.is_file() {
        return Err("Path is not a file".into());
    }    

    if config
        .tracking
        .file_map
        .values()
        .any(|v| v == &abs_path)
    {
        return Err("Already Tracked with different name".into());
    }
    if config
        .tracking
        .file_map
        .contains_key(name.as_str())
    {
        return Err("Already Tracked".into());
    }

    
    config.tracking.file_map.insert(name, abs_path);
    save_config(&config)?;
    
    Ok(())
}


/// Remove a file from the tracking list
pub fn _remove_tracking_file(name: String) -> Result<(), String> {
    let mut config = load_config()?;

    if config.tracking.file_map.remove(&name).is_none() {
        return Err("File not found in tracking list".into());
    }

    save_config(&config)?;

    Ok(())
}

/// List all tracked files
pub fn list_tracked_files() -> Result<(), String> {
    let config = load_config()?;

    if config.tracking.file_map.is_empty() {
        println!("No files are being tracked.");
        return Ok(());
    }

    for (name, path) in &config.tracking.file_map {
        ui::print_table (&name, &path.display().to_string(), None);
    }

    Ok(())
}
/// Check if a file is being tracked. 
pub fn is_tracked(name: &str) -> bool {
    if let Ok(config) = load_config() {
        config.tracking.file_map.contains_key(name)
    } else {
        false
    }
}
 
/// Returns the file system path associated with a tracked file alias.
///
/// # Arguments
///
/// * `name` - The alias of the tracked file.
///
/// # Returns
///
/// Returns `Ok(PathBuf)` with the file path if the alias is found, or an `Err` with a message if the alias is not tracked.
///
/// # Examples
///
/// ```
/// let path = get_path_from_alias("confsync").unwrap();
/// assert!(path.exists());
/// ```
pub fn get_path_from_alias(name: &str) -> Result<PathBuf, String> {
    let config = load_config()?;

    config
        .tracking
        .file_map
        .get(name)
        .cloned()
        .ok_or_else(|| format!("File {} is not being tracked", name))
}
