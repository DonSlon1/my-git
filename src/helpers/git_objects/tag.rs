use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::ops::Add;
use std::path::{Path, PathBuf};
use clap::builder::Str;
use ordermap::{OrderMap, OrderSet};
use sha1::digest::generic_array::arr;
use crate::helpers::git::GitRepo;
use crate::helpers::git_objects::git_object::GitObject;

#[derive(Debug)]
pub struct GitTag {
    fmt: Vec<u8>,
    data: Vec<u8>,
}

impl GitTag {
    pub fn new(data:Vec<u8>) -> Self {
        GitTag { fmt: b"tag".to_vec(), data }
    }
}

impl GitObject for GitTag {
    fn serialize(&self) -> String {
        self.data.iter()
            .filter(|&byte| {
                byte.is_ascii() && (byte.is_ascii_graphic() || byte.is_ascii_whitespace())
            })
            .map(|&byte| {
                byte as char
            }).collect::<String>()
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
    pub fn ref_resolve(&self, path: PathBuf) -> Option<String> {
        if !path.is_file() { return None; }
        
        let data = std::fs::read_to_string(path).unwrap().trim_end().to_string();
        if data.starts_with("ref: ") { 
            self.ref_resolve(data[5..].to_string().into())
        } else { 
            Some(data)
        }
    }

    pub fn ref_list(&self, path: Option<PathBuf>, prefix: String) -> std::io::Result<BTreeMap<String, String>> {
        let path = path.unwrap_or_else(|| self.repo_dir("refs".into(),true).unwrap());
        let mut ret = BTreeMap::new();

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_name = entry.file_name().into_string().unwrap();
            let can = entry.path();

            let new_prefix = if prefix.is_empty() {
                file_name.clone()
            } else {
                format!("{}/{}", prefix, file_name)
            };

            if can.is_dir() {
                let nested_refs = self.ref_list(Some(can), new_prefix)?;
                ret.extend(nested_refs);
            } else if let Some(sha) = self.ref_resolve(can) {
                ret.insert(new_prefix, sha);
            }
        }

        Ok(ret)
    }
}
