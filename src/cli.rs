use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "emd")]
#[command(about = "AWS resource explorer and Markdown documentation generator")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Update to the latest version
    Update,
}

pub fn run() -> Option<()> {
    let cli = Cli::parse();

    match cli.command? {
        Command::Update => {
            if let Err(e) = crate::update::perform_update() {
                eprintln!("Update failed: {}", e);
                std::process::exit(1);
            }
            Some(())
        }
    }
}
