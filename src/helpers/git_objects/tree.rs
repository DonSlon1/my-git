use std::any::Any;
use std::ops::Add;
use std::path::PathBuf;
use crate::helpers::git_objects::git_object::GitObject;
use crate::helpers::git_objects::tree_leaf::GitTreeLeaf;

#[derive(Debug, Clone)]
pub struct GitTree {
    fmt: Vec<u8>,
    data: Vec<u8>,
    leafs: Vec<GitTreeLeaf>
}

impl GitTree {
    pub fn new(data:Vec<u8>,leafs:Vec<GitTreeLeaf>) -> Self {
        GitTree { fmt: b"tree".to_vec(), data,leafs}
    }
    pub fn from_raw(raw: &[u8]) -> Self {
        let mut pos:usize = 0;
        let max = raw.len();
        let mut leafs = Vec::new();
        while pos < max {
            match GitTreeLeaf::new_from_raw(raw,Some(pos)) {
                None => {}
                Some(value) => {
                    pos = value.0;
                    leafs.push(value.1);
                }
            }
        }
        Self::new(raw.to_vec(),leafs)
    }
}

impl GitObject for GitTree {
    fn serialize(&self) -> String {
        let mut leafs = self.leafs.clone();
        leafs.sort_by(|a,b| a.clone().sort_keys().cmp(&b.clone().sort_keys()));
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
