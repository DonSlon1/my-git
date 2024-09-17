use crate::helpers::file::create_new_my_git;
use crate::helpers::git::GitRepo;
use crate::helpers::git_objects::commit::GitCommit;
use crate::helpers::git_objects::git_object::{AsAny, ObjectType};
use crate::helpers::git_objects::tree::GitTree;
use crate::helpers::git_objects::tree_leaf::GitTreeLeaf;
use crate::helpers::kvlm::kvlm_parse;
use crate::helpers::pager::display_with_pager;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Stdin, Write};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{UNIX_EPOCH, Duration};

pub fn init(path: String) {
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

pub fn add(path: String) {
    let git_repo = GitRepo::repo_find(path.into());
    match git_repo {
        None => {}
        Some(v) => {
            v.index_read();
        }
    }
}

pub fn cat_file(object_type: &ObjectType, object: &String) {
    let repo = GitRepo::repo_find(".".into());
    match repo {
        None => {
            eprintln!("No git repo find")
        }
        Some(v) => {
            std::io::stdout().write(&*v.cat_file(object.clone(), object_type.clone()).unwrap());
        }
    }
}

pub fn hash_obj(object_type: &ObjectType, path: &String, write: &bool) {
    let repo = match write {
        true => GitRepo::repo_find(".".into()),
        false => None,
    };
    let sha = GitRepo::hash_obj(repo.as_ref(), path.into(), object_type.clone());
    match sha {
        Ok(v) => println!("{}", v),
        Err(e) => eprintln!("{}", e),
    };
}

pub fn log(commit: String) {
    let repo = GitRepo::repo_find(".".into());
    match repo {
        None => {
            eprintln!("No git repo find")
        }
        Some(v) => {
            let mut data = "".to_string();
            v.log(
                v.obj_find(commit, None, None).unwrap(),
                &mut HashSet::new(),
                &mut data,
            );
            let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());

            let mut child = std::process::Command::new(&pager)
                .stdin(Stdio::piped())
                .spawn()
                .unwrap_or_else(|_| panic!("Failed to start pager: {}", pager));

            // Write the output to the pager
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(data.as_bytes())
                    .expect("Failed to write to pager");
            }

            // Wait for the pager process to exit
            child.wait().expect("Failed to wait on pager");
        }
    }
}

pub fn ls_tree(recursive: &bool, tree: &String) {
    let repo = match GitRepo::repo_find(".".into()) {
        None => {
            println!("No gi repo find");
            return;
        }
        Some(v) => v,
    };
    repo.ls_tree(tree, recursive, Some("".to_string()));
}

pub fn checkout(sha: String, path: PathBuf) {
    let repo = GitRepo::repo_find(".".into())
        .ok_or("No git repo found")
        .unwrap();

    let commit_obj = repo
        .object_read(sha)
        .map_err(|_| "Failed to read object")
        .unwrap();
    let commit = commit_obj
        .as_ref()
        .as_any()
        .downcast_ref::<GitCommit>()
        .ok_or("Not a commit")
        .unwrap();

    let tree_sha = commit
        .kvlm
        .get(&b"tree".to_vec())
        .and_then(|v| v.first())
        .ok_or("Commit doesn't contain a tree")
        .unwrap();

    let tree_sha_str = String::from_utf8(tree_sha.clone())
        .map_err(|_| "Invalid UTF-8 in tree SHA")
        .unwrap();
    let tree_obj = repo
        .object_read(tree_sha_str)
        .map_err(|_| "Failed to read tree object")
        .unwrap();

    let tree = tree_obj
        .as_ref()
        .as_any()
        .downcast_ref::<GitTree>()
        .ok_or("Tree in commit is not a tree")
        .unwrap();
    if path.clone().exists() {
        if !path.is_dir() {
            eprintln!(
                "Not a directory: {}",
                path.into_os_string().into_string().unwrap()
            );
            return;
        }
        if path.read_dir().iter().len() != 0 {
            //eprintln!("Not a directory: {:?}",path.into_os_string().into_string());
            return;
        }
    } else {
        std::fs::create_dir_all(path.clone()).unwrap()
    }
    repo.tree_checkout(Box::new(tree.clone()), std::fs::canonicalize(path).unwrap())
}

pub fn show_ref() {
    let repo = GitRepo::repo_find(".".into()).unwrap();
    let ref_list = repo.ref_list(None, "refs".to_string()).unwrap();
    for (key, value) in ref_list {
        println!("{} {}", value, key);
    }
}

pub fn tag(name: &Option<String>, create: &bool, object: &String, message: &Option<String>) {
    let repo = GitRepo::repo_find(".".into()).unwrap();
    match name {
        None => {
            let ref_list = repo
                .ref_list(
                    Some(repo.repo_dir("refs/tags".to_string(), true).unwrap()),
                    "refs".to_string(),
                )
                .unwrap();
            let mut tags = String::new();
            for (key, _value) in ref_list {
                let key: Vec<_> = key.rsplit('/').collect();
                if let Some(key) = key.first() {
                    tags.push_str(&format!("{}\n", key));
                }
            }
            display_with_pager(&*tags)
        }
        Some(name) => repo.create_tag(
            name,
            object.clone(),
            create.clone(),
            message.clone().as_ref(),
        ),
    }
}

pub fn rev_parse(name: &String) {
    let repo = GitRepo::repo_find(".".into()).unwrap();
    println!("{}", repo.obj_find(name.clone(), None, Some(true)).unwrap())
}

pub fn ls_files(verbose: bool) {
    let repo = GitRepo::repo_find(".".into()).unwrap();
    let index = repo.index_read();
    if verbose {
        println!("Index file format v{}, containing {} entries.",index.version.unwrap_or(0), index.entries.len())       
    }
    let mode_type_map: HashMap<u16, &str> = [
        (0b1000, "regular file"),
        (0b1010, "symlink"),
        (0b1110, "git link"),
    ].iter().cloned().collect();

    for e in &index.entries {
        println!("{}", e.name);
        if verbose {
            if let Some(mode_type_str) = mode_type_map.get(&e.mode_type) {
                println!("  {} with perms: {:o}", mode_type_str, e.mode_perms);
            }
            println!("  on blob: {}", e.sha);

            let ctime = UNIX_EPOCH + Duration::new(e.ctime.0 as u64, e.ctime.1);
            let mtime = UNIX_EPOCH + Duration::new(e.mtime.0 as u64, e.mtime.1);
            let ctime_secs = ctime.duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
            let mtime_secs = mtime.duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
            println!("  created: {}.{}, modified: {}.{}",
                     ctime_secs, e.ctime.1,
                     mtime_secs, e.mtime.1);

            println!("  device: {}, inode: {}", e.dev, e.ino);

            if let Ok(user) = get_user_by_uid(e.uid) {
                if let Ok(group) = get_group_by_gid(e.gid) {
                    println!("  user: {} ({})  group: {} ({})", 
                        user.name, e.uid, group.name, e.gid);
                }
            }

            println!("  flags: stage={} assume_valid={}", 
                e.flag_stage, e.flag_assume_valid);
        }
    }
}

fn get_user_by_uid(uid: u32) -> Result<User, &'static str> {
    let user = fs::metadata(format!("/proc/self/fd/0")).map_err(|_| "Failed to get user")?;
    Ok(User {
        name: user.uid().to_string(),
    })
}

fn get_group_by_gid(gid: u32) -> Result<Group, &'static str> {
    let group = fs::metadata(format!("/proc/self/fd/0")).map_err(|_| "Failed to get group")?;
    Ok(Group {
        name: group.gid().to_string(),
    })
}

struct User {
    name: String,
}

struct Group {
    name: String,
}