use std::collections::HashSet;
use std::io::{Stdin, Write};
use std::process::Stdio;
use crate::helpers::file::{create_new_my_git};
use crate::helpers::git::GitRepo;
use crate::helpers::git_objects::git_object::ObjectType;
use crate::helpers::git_objects::tree::GitTree;
use crate::helpers::git_objects::tree_leaf::GitTreeLeaf;
use crate::helpers::kvlm::kvlm_parse;

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
            let e =v.object_read(String::from("dd1cb88b72c47bfd55e6fa51cff67f75550bd735"));
            match e {
                Ok(ob) => {
                    println!("{:?}",GitTree::from_raw(&*ob.data()));
                }

                Err(e) => {println!("{}",e.to_string())}
            }
        }
    }
    
}

pub fn  cat_file(object_type: &ObjectType, object: &String) {
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
        false => None
    };
    let sha = GitRepo::hash_obj(repo,path.into(),object_type.clone());
    match sha {
        Ok(v) => println!("{}",v),
        Err(e) => eprintln!("{}",e)
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
            v.log(commit,&mut HashSet::new(), &mut data);
            let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());
            
            let mut child = std::process::Command::new(&pager)
                .stdin(Stdio::piped())
                .spawn()
                .unwrap_or_else(|_| panic!("Failed to start pager: {}", pager));
            
            // Write the output to the pager
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(data.as_bytes()).expect("Failed to write to pager");
            }

            // Wait for the pager process to exit
            child.wait().expect("Failed to wait on pager");
        }
    }
}