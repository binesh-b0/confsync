use directories::ProjectDirs;
use std::{fs, path::PathBuf};

// Copy tracked file
pub fn copy_file_to_repo(src: PathBuf, alias: &str, profile: &str) -> Result<(), String> {
    let project_dirs =
        ProjectDirs::from("", "", "confsync").expect("Failed to get project directories");
    let repo_path = project_dirs.data_dir().join(profile);

    // extract the file name from the path
    let file_name = src
        .file_name()
        .ok_or_else(|| "Failed to get file name".to_string())?
        .to_str()
        .ok_or_else(|| "Failed to convert file name to string".to_string())?;

    let dest = repo_path.join(alias).join(file_name);
    // create the directory if it doesn't exist
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    println!("Copying {} to {}", src.display(), dest.display());

    std::fs::copy(src, dest)
        .map(|_| ())
        .map_err(|e| format!("Failed to copy file: {}", e))
}
