use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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
            let config: Config = toml::from_str(&contents)
                .map_err(|e| format!("Failed to parse config file: {}", e))?;
            Ok(config)
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
