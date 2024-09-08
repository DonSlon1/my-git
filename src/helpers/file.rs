use crate::helpers::{config, git};
use clap::builder::Str;
use std::collections::{HashMap, HashSet};
use std::f32::consts::E;
use std::ffi::OsString;
use std::io;
use std::io::Write;
use std::path::PathBuf;

pub fn dir_exists(path: &str) -> bool {
    std::path::Path::new(path).is_dir()
}

pub fn dir_create_nested(path: &PathBuf) -> io::Result<()> {
    std::fs::create_dir_all(path)
}

pub fn get_exe_dir() -> String {
    std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

pub fn is_my_git_dir(mut path: PathBuf) -> bool {
    if !path.is_dir() {
        return false;
    }
    path.push(".git");
    if !path.is_dir() {
        return false;
    }

    let mut mandatory_files = HashSet::from(["config", "description", "HEAD"]);
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let dir_path = entry.path();
        if dir_path.is_file() {
            if let file_name = dir_path.file_name().unwrap().to_str().unwrap() {
                if mandatory_files.contains(&file_name) {
                    mandatory_files.remove(file_name);
                }
            }
        }
    }
    mandatory_files.len() == 0
}

pub fn create_new_my_git(mut path: PathBuf) -> Result<bool, String> {
    let get_repo = git::GitRepo::init(path.clone(), true);

    if get_repo.work_dir.exists() {
        if !get_repo.work_dir.is_dir() {
            return Err(format!("Path: {:?} is dir", get_repo.work_dir));
        } else if get_repo.git_dir.read_dir().iter().len() != 0 {
            return Err(format!("Dir: {:?} is not empty", get_repo.git_dir));
        }
    } else {
        match std::fs::create_dir_all(path) {
            Ok(_) => {}
            Err(e) => {
                return Err(e.to_string());
            }
        }
    }

    get_repo.repo_dir("branches".to_string(), true)?;
    get_repo.repo_dir("objects".to_string(), true)?;
    get_repo.repo_dir("refs/tags".to_string(), true)?;
    get_repo.repo_dir("refs/heads".to_string(), true)?;

    let conf = config::get_default_conf();
    let conf_file = get_repo.repo_file("config".to_string(), true)?;

    match conf.write(conf_file) {
        Ok(_) => {}
        Err(e) => return Err(e.to_string()),
    }

    match std::fs::File::create(get_repo.repo_file("description".to_string(), false)?) {
        Ok(mut f) => {
            f.write(
                "Unnamed repository; edit this file 'description' to name the repository.\n"
                    .as_ref(),
            )
            .unwrap();
        }
        Err(e) => return Err(e.to_string()),
    }

    match std::fs::File::create(get_repo.repo_file("HEAD".to_string(), false)?) {
        Ok(mut f) => {
            f.write("ref: refs/heads/master\n".as_ref()).unwrap();
        }
        Err(e) => return Err(e.to_string()),
    }

    Ok(true)
}
