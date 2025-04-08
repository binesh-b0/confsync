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