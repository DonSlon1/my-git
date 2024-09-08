mod commands;
pub mod helpers;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[clap(default_value_t = helpers::file::get_exe_dir())]
        path: String,
    },
    Add {
        #[clap(default_value_t = helpers::file::get_exe_dir())]
        path: String,
    },
    Checkout {
        #[clap(default_value_t = helpers::file::get_exe_dir())]
        path: String,
    },
    Commit {
        #[clap(default_value_t = helpers::file::get_exe_dir())]
        path: String,
    },
    Rm {
        #[clap(default_value_t = helpers::file::get_exe_dir())]
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add { path } => {
            commands::init::add(path.clone());
        }
        Commands::Init { path } => {
            commands::init::init(path.clone());
        }
        Commands::Checkout { path } => {
            println!("'myapp checkout' was used");
        }
        Commands::Commit { path } => {
            println!("'myapp commit' was used");
        }
        Commands::Rm { path } => {
            println!("'myapp rm' was used");
        }
    }
}
