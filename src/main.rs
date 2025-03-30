use clap::Parser;

mod cli;

use cli::Cli;

fn main(){
    let cli = Cli::parse();
    match cli.command {
        cli::Commands::Init {repo_url, local, force} => {
            println!("init command");
            println!("repo_url: {:?}", repo_url);
            println!("local: {}", local);
            println!("force: {}", force);
        },
        cli::Commands::Add { path,alias, encrypt } => {
            println!("add command");
            println!("path: {}", path);
            println!("alias: {:?}", alias);
            println!("encrypt: {}", encrypt);
        },
        _ => {
            println!("other command");
        }
    }
}