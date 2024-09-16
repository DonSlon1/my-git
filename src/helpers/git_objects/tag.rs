use crate::helpers::git::GitRepo;
use crate::helpers::git_objects::git_object::GitObject;
use crate::helpers::kvlm::{kvlm_parse, kvlm_serialize};
use clap::builder::Str;
use ordermap::{OrderMap, OrderSet};
use sha1::digest::generic_array::arr;
use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::hash::RandomState;
use std::ops::Add;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct GitTag {
    fmt: Vec<u8>,
    data: Vec<u8>,
    pub kvlm: OrderMap<Vec<u8>, Vec<Vec<u8>>, RandomState>,
}

impl GitTag {
    pub fn new(data: Vec<u8>) -> Self {
        let mut kvlm;
        if data.len() > 0 {
            let borrowed_kvlm = kvlm_parse(&data, None, None);

            // Convert borrowed OrderMap to owned OrderMap
            kvlm = borrowed_kvlm
                .into_iter()
                .map(|(k, v)| (k.to_vec(), v))
                .collect();
        } else { 
           kvlm = OrderMap::new(); 
        }
        GitTag {
            fmt: b"tag".to_vec(),
            data,
            kvlm,
        }
    }
}

impl GitObject for GitTag {
    fn serialize(&self) -> String {
        kvlm_serialize(&self.kvlm)
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
        if !path.is_file() {
            return None;
        }

        let data = std::fs::read_to_string(path)
            .unwrap()
            .trim_end()
            .to_string();
        if data.starts_with("ref: ") {
            self.ref_resolve(data[5..].to_string().into())
        } else {
            Some(data)
        }
    }

    pub fn ref_list(
        &self,
        path: Option<PathBuf>,
        prefix: String,
    ) -> std::io::Result<BTreeMap<String, String>> {
        let path = path.unwrap_or_else(|| self.repo_dir("refs".into(), true).unwrap());
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

    pub fn create_tag(&self, name: &String, reference: String, create_tag_object: bool, message: Option<&String>) {
        let sha = self.obj_file(reference,"tags".to_string(),None);
        
        if !create_tag_object { 
            self.ref_create(&"tags/".to_string().add(name),&sha);
            return;
        }
        
        let mut tag = GitTag::new(vec![]);
        tag.kvlm = OrderMap::new(); 
        tag.kvlm.insert(b"object".to_vec(),vec![sha.as_bytes().to_vec()]);
        tag.kvlm.insert(b"type".to_vec(),vec![b"commit".to_vec()]);
        tag.kvlm.insert(b"tag".to_vec(),vec![name.as_bytes().to_vec()]);
        tag.kvlm.insert(b"tagger".to_vec(),vec![b"Wyag <wyag@example.com>".to_vec()]);
        if let Some(message) = message {
            tag.kvlm.insert(b"None".to_vec(),vec![message.as_bytes().to_vec()]);
        }
        let tag_sha = GitRepo::object_write(Some(self), Box::new(tag)).unwrap();
        self.ref_create(&"/tags/".to_string().add(name),&tag_sha);
    }

    pub fn ref_create(&self, ref_name: &String, sha: &String) {
        let path = self
            .repo_file("refs".to_string().add(ref_name), true)
            .unwrap();
        std::fs::write(path, sha.clone().add("\n")).unwrap();
    }
}
