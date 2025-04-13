use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "confSync", disable_version_flag = true)]
#[command(about = "backup and sync configuration files")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>, // Made optional to allow running without a subcommand
    /// Verbose output
    #[arg(short='V', long, global = true)]
    pub verbose: bool,
    /// Hide warnings and errors
    #[arg(short, long, global = true)]
    pub quiet: bool,
    /// Set profile [default] 
    #[arg(short = 'P', global = true)]
    pub profile: Option<String>,
    /// show paths used
    #[arg(short,long, global = true)]
    pub paths: bool,
    /// Version
    #[arg(short = 'v', long, global = true)]
    pub version: bool,

}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize confsync with new repository
    Init {
        /// URL of the remote repository 
        repo_url: Option<String>,

        /// Use local repo instead of remote
        #[arg(long)]
        local: bool,

        /// Force overwrite existing configuration
        #[arg(long)]
        force: bool,
    },

    /// Track a configuration file for backup
    Add {
        /// Unique name / alias for the configuration file
        name:String,
        /// Path to the configuration file to track
        path: String,
    },

    /// Untrack a configuration file
    Remove {
        /// Untrack file in path
        #[arg(required_unless_present = "alias")]
        path: Option<String>,
        /// Untrack name
        #[arg(required_unless_present = "path")]
        alias: Option<String>,
    },

    /// Commit changes and push to the repo 
    Backup {
        /// file to be backed up. [default: all]
        alias: Option<String>,

        /// Custom commit message
        #[arg(short, long)]
        message: Option<String>,

        /// Push changes to the remote repository
        #[arg(long)]
        push: bool,

        /// Force push changes to the remote repository
        #[arg(short, long)]
        force: bool,

    },

    /// Restore a configuration file 
    Restore {
        /// Commit hash or tag (e.g., @latest)
        target: String,

        #[arg(short, long)]
        dry_run: bool,

        /// Overwrite if file exists
        #[arg(short, long)]
        overwrite: bool,
    },

    /// Show backup history
    List {

        /// Show tracked files
        #[arg(short, long)]
        tracked: bool,

        ///  history for an alias    
        #[arg(short, long, required_unless_present = "tracked")]
        alias: Option<String>,

    },

    /// Daemon mode for auto backup (Phase 2)
    Watch {
        /// Delay before triggering the backup (ms)
        #[arg(long, default_value_t = 2000)]
        debounce: u64,
    },

    /// Manage multiple profiles (Phase 2)
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    /// View and edit the confSync settings
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Forward git commands
    Git {
        /// Git command to execute
        #[arg(required = true)]
        args: Vec<String>,
    },

    /// Delete the local and/or remote repository or the configuration file
    Delete {
        #[command(subcommand)]
        target: DeleteTarget,
    },

    /// Show changed/untracked files
    Status,
}

#[derive(Subcommand, Debug)]
pub enum DeleteTarget {
    /// Delete the configuration file
    Config {
        /// Confirm deletion operation. ** There is no undo! **
        #[arg(long, required = true)]
        force: bool,
    },
    /// Delete the local repository
    Local {
        /// Confirm deletion operation. ** There is no undo! **
        #[arg(long, required = true)]
        force: bool,
    },
    /// Delete the remote repository
    Remote {
        /// Confirm deletion operation. ( only deletes main branch ) ** There is no undo! **
        #[arg(long, required = true)]
        force: bool,
    },
    /// Delete everything
    All {
        /// Confirm deletion operation. ** There is no undo! **
        #[arg(long, required = true)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProfileCommands {
    Create { name: String, repo_url: Option<String> },
    List,
    Switch { name: String },
    Delete { name: String, #[arg(long)] force: bool },
    Rename { old_name: String, new_name: String },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show the current config
    Show,
    /// Edit the config file
    Edit,
}
