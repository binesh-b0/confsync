use indicatif::{ProgressBar, ProgressStyle};
use colored::*;

#[allow(dead_code)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
    Default,
    Git,
}

pub fn printer(message: &str, message_type: MessageType) {
    match message_type {
        MessageType::Info => println!("{}", message.blue()),
        MessageType::Success => println!("{}", message.green()),
        MessageType::Warning => println!("{}", message.yellow()),
        MessageType::Error => println!("{}", message.red()),
        MessageType::Default => println!("{}", message),
        MessageType::Git => {
            // print a small chip showing git, then rest as normal
            let git_chip = "git".blue().bold();
            let replaced_message = message.replace("git", "");
            println!(
                "{} \n{}",
                git_chip.italic(),
                replaced_message
                    .green()
                    .bold()
                    
                    .bright_black()
                    .italic()
            );
        },
    }
}


/// Styles a file system path.
pub fn style_path(path: &str) -> String {
    // Split the path into its components.
    let segments: Vec<&str> = path.split('/').collect();
    let mut styled = String::new();

    for (i, segment) in segments.iter().enumerate() {
        // Append the separator styled in bright black.
        if i > 0 {
            styled.push_str(&"/".bright_black().to_string());
        }
        // Style the file name (last segment) bold; directories in italic.
        if i == segments.len() - 1 {
            styled.push_str(&segment.cyan().bold().to_string());
        } else {
            styled.push_str(&segment.cyan().italic().to_string());
        }
    }

    styled
}

/// Simple helper to split a string into lines of at most `width` characters.
fn wrap_text(s: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for c in s.chars() {
        current.push(c);
        if current.chars().count() >= width {
            lines.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Prints in a table format with 2 columns.
/// If the name or the value exceeds the allotted width, it wraps onto a new line
pub fn print_table(name: &str, value: &str, style: Option<&str>) {
    let style = style.unwrap_or("default");
    const NAME_WIDTH: usize = 15;
    const VALUE_WIDTH: usize = 40;

    // Wrap the name and value text to handle edge cases of very long strings.
    let name_lines = wrap_text(name, NAME_WIDTH);
    let value_lines = wrap_text(value, VALUE_WIDTH);
    let num_lines = std::cmp::max(name_lines.len(), value_lines.len());

    // Closure to apply style to the name.
    let style_name = |s: &str| -> String {
        match style {
            "default" => s.blue().to_string(),
            "git" => s.blue().bold().to_string(),
            _ => s.normal().to_string(),
        }
    };

    // Closure to apply path styling to the value.
    let style_value = |s: &str| -> String { style_path(s).to_string() };

    for i in 0..num_lines {
        let name_part = if i < name_lines.len() {
            style_name(&name_lines[i])
        } else {
            "".to_string()
        };
        let value_part = if i < value_lines.len() {
            style_value(&value_lines[i])
        } else {
            "".to_string()
        };

        // The first line prints the name column padded to a fixed width.
        // Subsequent lines print an empty first column to maintain table format.
        if i == 0 {
            println!("{:<width$} {}", name_part, value_part, width = NAME_WIDTH);
        } else {
            println!("{:<width$} {}", "", value_part, width = NAME_WIDTH);
        }
    }
}


// pub fn _print_progress_bar(message: &str) -> ProgressBar {
//     let pb = ProgressBar::new(100);
//     pb.set_style(
//         ProgressStyle::with_template("{msg} [{bar:40.cyan/blue}] {percent}%")
//             .unwrap()
//             .progress_chars("##-"),
//     );
//     pb.set_message(message);
//     pb
// }

pub fn _run_with_spinner<F, T>(message: &str, task: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    let spinner = ProgressBar::new_spinner();
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠁", "⠂", "⠄", "⠂"])
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    let result = task();

    match &result {
        Ok(_) => spinner.finish_with_message(format!("{} {}", "✓".green().bold(), message)),
        Err(e) => spinner.finish_with_message(format!("{} {}: {}", "✗".red().bold(), message, e)),
    }

    result
}
