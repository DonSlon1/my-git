use std::collections::HashSet;
use crate::helpers::config;
use crate::helpers::file::is_my_git_dir;
use configparser::ini::Ini;
use std::path::PathBuf;
use crate::helpers::git_objects::git_object::GitObject;

#[derive(Debug)]
pub struct GitRepo {
    pub work_dir: PathBuf,
    pub git_dir: PathBuf,
    pub config: Option<Ini>,
}

impl GitRepo {
    pub fn init(path: PathBuf, force: bool) -> Self {
        let work_dir = path.clone();
        let git_dir = path.clone().join(".git");

        if !is_my_git_dir(path.clone().into()) && !force {
            eprintln!(
                "Dir: {:?} is not a git repository need to run mygit init",
                path.to_str()
            );
            std::process::exit(1);
        }

        let cf = match config::read_conf(git_dir.clone().join("config")) {
            Ok(o) => Some(o),
            Err(e) => {
                if !force {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
                None
            }
        };

        if !force && cf.is_some() {
            let version = cf.clone().unwrap().get("core", "repositoryformatversion");
            match version {
                None => {
                    eprintln!("missing mandatory attribute section: core attribute: repositoryformatversion");
                    std::process::exit(1);
                }
                Some(value) => {
                    if !value.eq("0") {
                        eprintln!("unsupported version");
                        std::process::exit(1);
                    }
                }
            }
        }

        let git_repo = Self {
            work_dir,
            git_dir,
            config: cf,
        };

        git_repo
    }

    pub fn repo_find(path: PathBuf) -> Option<GitRepo> {
        let git_path = path.clone().join(".git");
        if git_path.exists() {
            return Some(GitRepo::init(path, false));
        }

        let parent = match path.parent() {
            None => {
                return None;
            }
            Some(v) => v,
        };
        if path.eq(parent) {
            return None;
        }

        GitRepo::repo_find(parent.to_path_buf())
    }

    /// Returns path from git directory and file in it
    ///
    pub fn repo_path(&self, file_path: String) -> PathBuf {
        let mut git_dir = self.git_dir.to_owned();
        git_dir.push(file_path);
        git_dir
    }

    pub fn repo_dir(&self, file_path: String, mkdir: bool) -> Result<PathBuf, String> {
        let path = self.repo_path(file_path.clone());
        if path.exists() {
            if path.is_dir() {
                return Ok(path);
            } else {
                return Err("Path is not a dir".to_string());
            }
        }

        if mkdir {
            match std::fs::create_dir_all(path.clone()) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e.to_string());
                }
            }
        }
        Ok(path)
    }

    pub fn repo_file(&self, file_path: String, mkdir: bool) -> Result<PathBuf, String> {
        let mut path: PathBuf = file_path.clone().into();
        path.pop();

        if self
            .repo_dir(path.to_str().unwrap().to_string(), mkdir)
            .is_ok()
        {
            return Ok(self.repo_path(file_path.to_string()));
        }
        Err("Try that again late".to_string())
    }
    
    pub fn log(&self, sha: String, mut seen: HashSet<String>) {
        if seen.contains(&sha) { return; }
        seen.insert(sha.clone());
        
        let object = match self.object_read(sha) {
            Ok(v) => v,
            Err(e) => {
                println!("{}",e);
                return;
            }
        };
    }
}
