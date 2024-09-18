use crate::helpers::git::GitRepo;
use crate::helpers::git_objects::git_object::{AsAny, GitObject, ObjectType};
use crate::helpers::git_objects::tree_leaf::GitTreeLeaf;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct GitTree {
    fmt: Vec<u8>,
    data: Vec<u8>,
    leafs: Vec<GitTreeLeaf>,
}

impl GitTree {
    pub fn new(data: Vec<u8>, leafs: Vec<GitTreeLeaf>) -> Self {
        GitTree {
            fmt: b"tree".to_vec(),
            data,
            leafs,
        }
    }
    pub fn from_raw(raw: &[u8]) -> Self {
        let mut pos: usize = 0;
        let max = raw.len();
        let mut leafs = Vec::new();
        while pos < max {
            match GitTreeLeaf::new_from_raw(raw, Some(pos)) {
                None => {}
                Some(value) => {
                    pos = value.0;
                    leafs.push(value.1);
                }
            }
        }
        Self::new(raw.to_vec(), leafs)
    }
}

impl GitObject for GitTree {
    fn serialize(&self) -> String {
        let mut leafs = self.leafs.clone();
        leafs.sort_by(|a, b| a.clone().sort_keys().cmp(&b.clone().sort_keys()));
        let mut output = String::new();
        for leaf in leafs {
            output.push_str(&*leaf.mode);
            output.push(' ');
            output.push_str(&leaf.clone().sort_keys());
            output.push('\x00');
            if let Ok(sha_bytes) = hex::decode(&leaf.sha) {
                output.push_str(&String::from_utf8(sha_bytes).unwrap_or("".to_string()))
            }
        }
        output
    }

    fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn deserialize(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn format(&self) -> Vec<u8> {
        self.fmt.clone()
    }
}

impl GitRepo {
    pub fn ls_tree(&self, tree: &String, recursive: &bool, prefix: Option<String>) {
        let sha = self
            .obj_find(tree.clone(), Some("tree".to_string()), None)
            .unwrap();
        let prefix = prefix.unwrap_or("".to_string());
        let obj = match self.object_read(sha) {
            Ok(v) => v,
            Err(_) => {
                return;
            }
        };
        let obj = match obj.as_ref().as_any().downcast_ref::<GitTree>() {
            None => {
                println!("{}", "Object is not a tree".to_string());
                return;
            }
            Some(v) => v.clone(),
        };
        for leaf in obj.leafs {
            let mut mode_type;
            if leaf.mode.len() == 5 {
                mode_type = &leaf.mode[0..1];
            } else {
                mode_type = &leaf.mode[0..2];
            }

            match mode_type {
                "04" => mode_type = "tree",
                "10" => mode_type = "blob",
                "12" => mode_type = "blob",
                "16" => mode_type = "commit",
                _ => {
                    println!("Unsoported leaf type: {}", leaf.mode)
                }
            }
            if !(*recursive && mode_type == "tree") {
                println!(
                    "{:0>6} {} {}\t{}",
                    leaf.mode,
                    mode_type,
                    leaf.sha,
                    Path::new(&prefix).join(&leaf.path).display()
                );
            } else {
                let new_prefix = Path::new(&prefix).join(&leaf.path).display().to_string();
                self.ls_tree(&leaf.sha, recursive, Some(new_prefix));
            }
        }
    }
    pub fn tree_checkout(&self, tree: Box<dyn GitObject>, path: PathBuf) {
        let tree = match tree.as_ref().as_any().downcast_ref::<GitTree>() {
            None => {
                eprintln!("Tree obj is not a tree");
                return;
            }
            Some(v) => v.clone(),
        };
        for leaf in tree.leafs {
            let obj = self.object_read(leaf.sha).unwrap();
            let mut dest = path.clone();
            dest.push(leaf.path);

            if obj.format() == b"tree".to_vec() {
                std::fs::create_dir_all(dest.clone()).unwrap();
                self.tree_checkout(obj, dest)
            } else if obj.format() == b"blob".to_vec() {
                let mut file = File::create(dest).unwrap();
                file.write_all(&*obj.data()).unwrap();
            }
        }
    }
    pub fn to_hash_map(&self, reference: String, prefix: Option<String>) -> HashMap<String, String> {
        let prefix = prefix.unwrap_or("".to_string());
        let mut ret = HashMap::new();
        let tre_sha = match self.obj_find(reference.clone(),Some("tree".to_string()),None) {
            Ok(v) => {v}
            Err(_) => {return ret}
        };
        let git_object = self.object_read(tre_sha.clone()).unwrap();
        let tree = match git_object.as_ref().as_any().downcast_ref::<GitTree>() {
            None => {return ret}
            Some(v) => v,
        };

        for leaf in tree.leafs.clone() {
            let full_path = Path::new(&prefix).join(leaf.path.clone());
            
            let is_sub_tree = leaf.mode.starts_with("04");
            if is_sub_tree { 
                ret.extend(self.to_hash_map(leaf.sha,Some(full_path.to_str().unwrap().to_string())));
            } else { 
                ret.insert(full_path.to_str().unwrap().to_string(),GitRepo::hash_obj(None,full_path,ObjectType::Blob).unwrap());
            }
        }
        
        
        ret
    }
}
