use crate::helpers::git::GitRepo;
use clap::builder::Str;
use std::fmt::format;
use std::fs::File;
use std::path::PathBuf;
use sha1::{Digest, Sha1Core};
use sha1::digest::{DynDigest, Update};
use sha1::digest::core_api::CoreWrapper;
use zune_inflate::errors::InflateDecodeErrors;
use crate::helpers::git_objects::blob::GitBlob;
use crate::helpers::git_objects::commit::GitCommit;
use crate::helpers::git_objects::tag::GitTag;
use crate::helpers::git_objects::tree::GitTree;

pub trait GitObject {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(&self) -> Vec<u8>;
    fn fmt(&self) -> &[u8];
}


impl GitRepo {
    /// Read object sha from Git repository repo.  Return a
    /// GitObject whose exact type depends on the object.
    pub fn object_read(&self, sha: String) -> Result<Box<dyn GitObject>,String> {
        let sha_split = sha.split_at(2);
        //let path = self.repo_file(format!("objects/{}/{}", sha_split.0, sha_split.1), false)?;
        let path:PathBuf = "/home/lukas/Nero/.git/objects/03/2464fb6ec40a523899b8c8a593242f3108a420".into();
        
        if !path.is_file() { 
            return Err(format!("Path: {:?} is not a file",path))
        }
        
        let data = match std::fs::read(path) {
            Ok(v) => v,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        let raw = match zune_inflate::DeflateDecoder::new(&*data).decode_zlib() {
            Ok(v) => v,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        if let Some(x) = raw.iter().position(|&b| b == b' ') {
            let fmt = &raw[0..x];

            if let Some(y) = raw.iter().position(|&b| b == b'\x00') {
                let size_str = std::str::from_utf8(&raw[x + 1..y])
                    .map_err(|_| "Invalid UTF-8 in size field")?;
                let size: usize = size_str
                    .parse()
                    .map_err(|_| "Invalid number in size field")?;

                if size != raw.len() - y - 1 {
                    return Err(format!("Malformed object {}: bad length", sha));
                }

                let data = &raw[y + 1..];

                let object: Box<dyn GitObject> = match fmt {
                    b"commit" => {
                        Box::new(GitCommit::new(Vec::from(data)))
                    }
                    b"tree" => {
                        Box::new(GitTree::new(Vec::from(data)))
                    }
                    b"tag" => {
                        Box::new(GitTag::new(Vec::from(data)))
                    }
                    b"blob" => Box::new(GitBlob::new(Vec::from(data))),
                    _ => {
                        return Err(format!(
                            "Unknown type {} for object {}",
                            std::str::from_utf8(fmt).unwrap_or("<invalid UTF-8>"),
                            sha
                        ));
                    }
                };
                Ok(object)
            } else {
                Err("Null byte not found".into())
            }
        } else {
            Err("Space not found".into())
        }
    }
    
    pub fn object_write(&self, object: Box<dyn GitObject>) -> Result<String,String> {
        let data = object.serialize();
        let size_str = data.len().to_string();
        let size_bytes = size_str.as_bytes();

        let mut result = Vec::new();
        result.extend_from_slice(object.fmt());
        result.push(b' ');
        result.extend_from_slice(size_bytes);
        result.push(b'\x00');
        result.extend_from_slice(&*data);
        
        let mut hasher = sha1::Sha1::new();
        let data : &[u8] = &*result;
        Digest::update(&mut hasher, data);
        let hash = hasher.finalize();
        let sha:String  = hash.iter().map(|b| format!("{:02x}", b)).collect();
        
        // write data to file
        let split_sha = sha.split_at(2);
        let path = self.repo_file(format!("objects/{}/{}",split_sha.0,split_sha.1),true)?;
        
        if !path.exists() {
            match std::fs::write(path,zune_inflate::DeflateEncoder::new(data).encode_zlib()) {
                Ok(_) => {}
                Err(e) => {
                    return Err(format!("{}",e.to_string()));
                }
            }
        } else {
            println!("existuje");
        }

        Ok(sha)
    }
}
