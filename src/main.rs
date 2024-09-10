mod commands;
pub mod helpers;
use clap::{Parser, Subcommand};
use commands::commands::{add, cat_file, init};
use helpers::git_objects::git_object::ObjectType;

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
    CatFile {
        object_type: ObjectType,
        object: String
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add { path } => {
            add(path.clone());
        }
        Commands::Init { path } => {
            init(path.clone());
        }
        Commands::Checkout { path: _path } => {
            println!("'myapp checkout' was used");
        }
        Commands::Commit { path: _path } => {
            println!("'myapp commit' was used");
        }
        Commands::Rm { path: _path } => {
            println!("'myapp rm' was used");
        }
        Commands::CatFile { object_type,object } => {
            cat_file(object_type,object);
        }
    }
}
