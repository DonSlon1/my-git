use crate::helpers::file::{create_new_my_git, is_my_git_dir};
use crate::helpers::git::GitRepo;

pub fn init(mut path: String) {
    let r = create_new_my_git(path.into());
    match r {
        Ok(_) => {
            println!("git dri was successfully created")
        }
        Err(e) => {
            eprintln!("{}", e.to_string());
            std::process::exit(1)
        }
    }
}

pub fn add(mut path: String) {
    let git_repo = GitRepo::repo_find(path.into());
    match git_repo {
        None => {}
        Some(v) => {
            println!("{:?}",v)
        }
    }
    
}