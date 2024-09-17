mod commands;
pub mod helpers;

use std::path::PathBuf;
use crate::commands::commands::{check_git_ignore, checkout, ls_files, ls_tree, rev_parse, show_ref, tag};
use clap::{Parser, Subcommand};
use commands::commands::{add, cat_file, hash_obj, init, log};
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
        commit: String,
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
        object: String,
    },
    HashObject {
        path: String,
        #[clap(short, action)]
        write: bool,
        #[clap(short = 't', action,value_enum, default_value_t=ObjectType::Blob)]
        object_type: ObjectType,
    },
    Log {
        #[clap(default_value = "HEAD")]
        commit: String,
    },
    LsTree {
        #[clap(short)]
        recursive: bool,
        tree: String,
    },
    Tag {
        name: Option<String>,
        #[clap(short = 'a')]
        create: bool,
        #[clap(default_value = "HEAD")]
        object: String,
        #[clap(short, long)]
        message: Option<String>,
    },
    ShowRef,
    RevParse {
        name: String,
    },
    LsFiles {
        #[clap(short, long)]
        verbose: bool,
    },
    CheckIgnore {
        paths: Vec<PathBuf>
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
        Commands::Checkout { commit, path } => {
            checkout(commit.clone(), path.into());
        }
        Commands::Commit { path: _path } => {
            println!("'myapp commit' was used");
        }
        Commands::Rm { path: _path } => {
            println!("'myapp rm' was used");
        }
        Commands::CatFile {
            object_type,
            object,
        } => {
            cat_file(object_type, object);
        }
        Commands::HashObject {
            object_type,
            path,
            write,
        } => hash_obj(object_type, path, write),
        Commands::Log { commit } => {
            log(commit.clone());
        }
        Commands::LsTree { recursive, tree } => ls_tree(recursive, tree),
        Commands::Tag {
            name,
            create,
            object,
            message,
        } => tag(name, create, object, message),
        Commands::ShowRef => show_ref(),
        Commands::RevParse { name } => rev_parse(name),
        Commands::LsFiles { verbose } => ls_files(*verbose),
        Commands::CheckIgnore {paths} => check_git_ignore(paths.clone())
    }
}
