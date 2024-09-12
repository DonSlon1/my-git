use std::collections::HashSet;
use std::io::Write;
use ordermap::OrderMap;
use crate::helpers::file::{create_new_my_git};
use crate::helpers::git::GitRepo;
use crate::helpers::git_objects::git_object::ObjectType;
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
/*    let git_repo = GitRepo::repo_find(path.into());
    match git_repo {
        None => {}
        Some(v) => {
            let e =v.object_read(String::from("001b3da827e2c31c716396bea874b0d8d15d1a6e"));
            match e {
                Ok(ob) => {
                    println!("{:?}",GitRepo::object_write(Some(v),ob));
                }

                Err(e) => {println!("{}",e.to_string())}
            }
        }
    }
*/    let data = 
        "tree 29ff16c9c14e2652b22f8b78bb08a5a07930c147
parent 206941306e8a8af65b66eaaaea388a7ae24d49a0
author Thibault Polge <thibault@thb.lt> 1527025023 +0200
committer Thibault Polge <thibault@thb.lt> 1527025044 +0200

Create first draft";
    println!("{:?}",kvlm_parse(data.as_bytes(),None,None));
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
            v.log(commit,HashSet::new());
        }
    }
}