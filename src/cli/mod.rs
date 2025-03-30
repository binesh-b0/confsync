use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name="confSync")]
#[command(about = "backup and sync configuration files")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,


    #[arg(short, long,global=true)]
    pub verbose: bool,

    /// Path to the config file
    #[arg(short, long, global=true)]
    pub config_path: Option<String>,

    #[arg(short, long, global=true)]
    pub profile: Option<String>,

}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new repository
    Init {
        /// URL of the remote repository 
        repo_url: Option<String>,

        /// Use local repo instead of remote
        #[arg(long)]
        local: bool,

        /// Force overwrite existing repo
        #[arg(long)]
        force: bool,
    },

    /// Track a configuration file for backup
    Add {
        path: String,
        /// Alias for the configuration file
        #[arg(long)]
        alias: Option<String>,
        /// Encrypt the configuration file (Phase 2)
        #[arg(long)]
        encrypt: bool,
    },

    /// Commit changes and push to the repo 
    Backup {
        /// Custom commit message
        #[arg(short, long)]
        message: Option<String>,

        /// Push changes to the remote repository
        #[arg(short, long)]
        push: bool,

        /// Force push changes to the remote repository
        #[arg(short, long)]
        force: bool,

        /// Show preview of changes
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Restore a configuration file 
    Restore {
        /// Commit hash or tag (e.g., @latest)
        target: String,

        #[arg(short, long)]
        dry_run: bool,

        /// Overwrite local files
        #[arg(short, long)]
        force: bool,    
    },

    /// Show backup history
    List {
        /// Compact view
        #[arg(short, long)]
        oneline: bool,

        /// Show changed files
        #[arg(short, long)]
        verbose: bool,
    },

    /// Daemon mode for auto backup (Phase 2)
    Watch {
        /// Delay beforte triggering the backup (ms)
        #[arg(long, default_value_t = 2000)] 
        debounce: u64,

    },

    /// Manage multiple profiles (Phase 2)
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    /// Manage encryption keys (Phase 2)
    Encrypt {
        #[command(subcommand)]
        command: EncryptCommands,
    },

    /// View and edit the confSync settings
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Show changed/ untracked files
    Status,

    /// Print Version
    Version,
}

#[derive(Subcommand, Debug)]
pub enum ProfileCommands {
    Create { name: String, repo_url: Option<String> },
    List,
    Switch { name: String },
    Delete { name: String, #[arg(long)] force : bool },
    Rename { old_name: String, new_name: String },
}

#[derive(Subcommand, Debug)]
pub enum EncryptCommands {
    Init,
    AddKey { pubkey_path: String },
    Rotate,
    Remove { path: String },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show the current config
    Show,
    /// Edit the config file
    Edit,
    /// Reset the config file
    Reset,
    Validate,
    Path,
}