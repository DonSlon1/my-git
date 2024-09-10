use crate::helpers::file::{create_new_my_git};
use crate::helpers::git::GitRepo;
use crate::helpers::git_objects::git_object::ObjectType;

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
            let e =v.object_read(String::from("001b3da827e2c31c716396bea874b0d8d15d1a6e"));
            match e {
                Ok(ob) => {
                    println!("{:?}",v.object_write(ob));
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
            println!("{}",v.cat_file(object.clone(), object_type.clone()).unwrap())
        }
    }
}