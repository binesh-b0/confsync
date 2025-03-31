use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
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
    pub files: Vec<FileMapping>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileMapping {
    pub src: String,
    pub alias: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let config_path = default_config_path()
            .unwrap_or_else(|| PathBuf::from("config.toml"));
        Self {
            storage: Storage {
                local: true,
                repo_url: "".to_string(),
            },
            tracking: Tracking {
                files: vec![
                    FileMapping {
                        src: config_path.to_string_lossy().to_string(),
                        alias: Some("confsync".to_string()),
                    }
                ],
            },
        }
    }
}

/// Get path to the user's config file
pub fn default_config_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "confsync") {
        let conf_dir = proj_dirs.config_dir();
        Some(conf_dir.join("config.toml"))
    }
    else {
        None
    }
}

/// Check if the config file exists : returns true if it does
pub fn check_config_exists() -> bool {
    default_config_path()
        .map_or(false,|path| path.is_file())
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
/// creating the directory if it doesn't exist
pub fn save_config(config: &Config) -> Result<(),String> {
    let path = match default_config_path() {
        Some(p) => p,
        None => return Err("Could not determine config path".into()),
    };

    // Ensure the directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let toml_string = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config :{e}"))?;

    fs::write(&path, toml_string)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// Delete the config file
pub fn delete_config() -> Result<(), String> {
    let path = match default_config_path() {
        Some(p) => p,
        None => return Err("Could not determine config path".into()),
    };

    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete config file: {}", e))?;
    } else {
        return Err("Config file does not exist".into());
    }

    Ok(())
}