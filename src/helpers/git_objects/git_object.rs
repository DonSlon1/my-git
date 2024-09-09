use crate::helpers::git::GitRepo;
use clap::builder::Str;
use std::fmt::format;
use std::fs::File;
use std::path::PathBuf;

pub trait GitObject {
    fn serialize(&self) -> String;
    fn deserialize(&self) -> String;
    fn fmt(&self) -> &[u8];
}

pub struct GitBlob<'a> {
    fmt: &'a [u8], 
    data: Vec<u8>,
}

impl<'a> GitBlob<'a> {
    pub fn new(data:Vec<u8>) -> Self {
        GitBlob { fmt: b"blob", data }
    }
}

impl GitObject for GitBlob<'_> {
    fn serialize(&self) -> String {
        todo!()
    }

    fn deserialize(&self) -> String {
        todo!()
    }

    fn fmt(&self) -> &[u8] {
        self.fmt
    }
}

impl GitRepo {
    /// Read object sha from Git repository repo.  Return a
    /// GitObject whose exact type depends on the object.
    pub fn object_read(&self, sha: String) -> Result<Box<dyn GitObject>,String> {
        let sha_split = sha.split_at(2);
        let path = self.repo_file(format!("objects/{}/{}", sha_split.0, sha_split.1), false)?;
        
        if !path.is_file() { 
            return Err(format!("Path: {:?} is not a file",path))
        }
        
        let data = match std::fs::read(path) {
            Ok(v) => v,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        let raw = zune_inflate::DeflateDecoder::new(&*data).decode_zlib();
        println!("raw: {:?}",raw);
        

        Ok(Box::new(GitBlob::new(data)))
    }
}
