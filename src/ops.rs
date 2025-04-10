use directories::ProjectDirs;
use std::{fs, io::{Read, Write}, path::{Path, PathBuf}};

use crate::ui::{self, printer};

/// helper fn to compare two files
fn compare_files(path1: &Path,path2: &Path) -> Result<bool, String> {
    let mut file1 = fs::File::open(path1)
        .map_err(|e| format!("Failed to open file1: {}", e))?;
    let mut file2 = fs::File::open(path2)
        .map_err(|e| format!("Failed to open file2: {}", e))?;

    let mut buf1 = [0u8;8192];
    let mut buf2 = [0u8;8192];

    loop {
        let n1 = file1.read(&mut buf1)
            .map_err(|e| format!("Failed to read file1: {}", e))?;
        let n2 = file2.read(&mut buf2)
            .map_err(|e| format!("Failed to read file2: {}", e))?;
        if n1 != n2 {
            return Ok(false);
        }
        if n1 == 0 {
            break;
        }
        if buf1[..n1] != buf2[..n2] {
            return Ok(false);
        }
    }
    Ok(true)
}


// Copy tracked file
pub fn copy_file_to_repo(src: PathBuf, alias: &str, profile: &str, force: bool) -> Result<(), String> {
    
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
    // compare the files
    if !force && dest.exists() {
        let src_meta = fs::metadata(&src)
            .map_err(|e| format!("Failed to get source file metadata: {}", e))?;
        let dest_meta = fs::metadata(&dest)
            .map_err(|e| format!("Failed to get destination file metadata: {}", e))?;

        if src_meta.len() == dest_meta.len() {
            if compare_files(&src, &dest)? {
                printer(format!("That one has a backup").as_str(), ui::MessageType::Success);
                write_log("info",
                    "COPY",
                    &format!("File already backedup {}", dest.display()), Some(profile.to_string()))?;
                return Ok(());
            }
        }
    }


    // Re-open source file to reset pointer for copying
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
    // append or create a new file => alias.cmt, to track backup time
    let cmt_file = repo_path.join(alias).join(format!("{}.cmt", file_name));
    let mut cmt_file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(cmt_file)
        .map_err(|e| format!("Failed to open comment file: {}", e))?;
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    writeln!(
        cmt_file,
        "[{}] {}",
        timestamp,
        src.display()
    )
    .map_err(|e| format!("Failed to write to comment file: {}", e))?;

    Ok(())
}

/// Read the cmt file: timestamp only
/// return the datetime of the commits in a list of strings
pub fn read_cmt(alias: &str, profile: &str) -> Result<Vec<String>, String> {
    let project_dirs =
        ProjectDirs::from("", "", "confsync").expect("Failed to get project directories");
    let repo_path = project_dirs.data_dir().join(profile);

    let cmt_file = repo_path
        .join(alias)
        .read_dir()
        .map_err(|e| format!("Failed to read directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .find(|entry| entry.path().extension().map_or(false, |ext| ext == "cmt"))
        .map(|entry| entry.path())
        .ok_or_else(|| "Failed to locate comment file with .cmt extension".to_string())?;
    if !cmt_file.exists() {
        return Err("Comment file does not exist".into());
    }

    let mut file = fs::File::open(cmt_file)
        .map_err(|e| format!("Failed to open comment file: {}", e))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read comment file: {}", e))?;

    let lines: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    Ok(lines)
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
