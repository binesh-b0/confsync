use directories::ProjectDirs;
use std::{fs, io::{Read, Write}, path::PathBuf};

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
    write_log("info", "COPY", &format!("Copying {} to {}", src.display(), dest.display()), Some(profile.to_string()))?;

    let file_size = fs::metadata(&src)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?
        .len();

    let mut src_file = fs::File::open(&src)
        .map_err(|e| format!("Failed to open source file: {}", e))?;
    let mut dest_file = fs::File::create(&dest)
        .map_err(|e| format!("Failed to create destination file: {}", e))?;

    let mut buffer = [0u8; 8192];
    let mut copied: u64 = 0;

    loop {
        let bytes_read = src_file
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read from source file: {}", e))?;
        if bytes_read == 0 {
            break;
        }

        dest_file
            .write_all(&buffer[..bytes_read])
            .map_err(|e| format!("Failed to write to destination file: {}", e))?;

        copied += bytes_read as u64;
        println!(
            "Progress: {:.2}%",
            (copied as f64 / file_size as f64) * 100.0
        );
    }

    Ok(())
}

/// write to log file
pub fn write_log(
    log_type: &str,
    action: &str,
    message: &str,
    profile: Option<String>,
) -> Result<(), String> {
    let profile_str = profile.as_deref().unwrap_or("default");

    let project_dirs =
        ProjectDirs::from("", "", "confsync").expect("Failed to get project directories");
    let log_path = project_dirs
        .data_dir()
        .join(profile_str)
        .join("log.txt");

    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    writeln!(
        file,
        "[{} | {}] {} => {}: {}",
        timestamp, profile_str, log_type, action, message
    )
    .map_err(|e| format!("Failed to write to log file: {}", e))?;

    Ok(())
}
