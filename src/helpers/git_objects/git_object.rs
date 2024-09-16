use crate::helpers::git::GitRepo;
use crate::helpers::git_objects::blob::GitBlob;
use crate::helpers::git_objects::commit::GitCommit;
use crate::helpers::git_objects::tag::GitTag;
use crate::helpers::git_objects::tree::GitTree;
use clap::ValueEnum;
use sha1::Digest;
use std::any::Any;
use std::fmt::Debug;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, ValueEnum)]
#[repr(u8)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl ObjectType {
    pub fn as_iter(&self) -> &[u8] {
        match self {
            ObjectType::Blob => b"blob",
            ObjectType::Tree => b"tree",
            ObjectType::Commit => b"commit",
            ObjectType::Tag => b"tag",
        }
    }
}

impl ToString for ObjectType {
    fn to_string(&self) -> String {
        match self {
            ObjectType::Blob => "blob".to_string(),
            ObjectType::Tree => "tree".to_string(),
            ObjectType::Commit => "commit".to_string(),
            ObjectType::Tag => "tag".to_string(),
        }
    }
}

impl FromStr for ObjectType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(ObjectType::Blob),
            "tree" => Ok(ObjectType::Tree),
            "commit" => Ok(ObjectType::Commit),
            "tag" => Ok(ObjectType::Tag),
            e => Err(format!("Invalid object type: {}", e)),
        }
    }
}
pub struct GitObjectFactory;

impl GitObjectFactory {
    pub fn new(object_type: ObjectType, data: Vec<u8>) -> Box<dyn GitObject> {
        match object_type {
            ObjectType::Blob => Box::new(GitBlob::new(data)),
            ObjectType::Tree => Box::new(GitTree::new(data, vec![])),
            ObjectType::Commit => Box::new(GitCommit::new(data)),
            ObjectType::Tag => Box::new(GitTag::new(data)),
        }
    }
}

pub trait GitObject: Any + Debug + AsAny {
    fn serialize(&self) -> String;
    fn deserialize(&self) -> Vec<u8>;
    fn format(&self) -> Vec<u8>;
    fn data(&self) -> Vec<u8>;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl GitRepo {
    /// Read object sha from Git repository repo.  Return a
    /// GitObject whose exact type depends on the object.
    pub fn object_read(&self, sha: String) -> Result<Box<dyn GitObject>, String> {
        let sha_split = sha.split_at(2);
        let path = self.repo_file(format!("objects/{}/{}", sha_split.0, sha_split.1), false)?;

        if !path.is_file() {
            return Err(format!("Path: {:?} is not a file", path));
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
                    b"commit" => Box::new(GitCommit::new(Vec::from(data))),
                    b"tree" => Box::new(GitTree::from_raw(data)),
                    b"tag" => Box::new(GitTag::new(Vec::from(data))),
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

    pub fn object_write(
        repo: Option<&GitRepo>,
        object: Box<dyn GitObject>,
    ) -> Result<String, String> {
        let data = object.serialize();
        let size_str = data.len().to_string();
        let size_bytes = size_str.as_bytes();

        let mut result = Vec::new();
        result.extend_from_slice(&object.format());
        result.push(b' ');
        result.extend_from_slice(size_bytes);
        result.push(b'\x00');
        result.extend_from_slice(data.as_ref());

        let mut hasher = sha1::Sha1::new();
        let data: &[u8] = &*result;
        Digest::update(&mut hasher, data);
        let hash = hasher.finalize();
        let sha: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
        if let Some(repo) = repo {
            // write data to file
            let split_sha = sha.split_at(2);
            let path = repo.repo_file(format!("objects/{}/{}", split_sha.0, split_sha.1), true)?;

            if !path.exists() {
                match std::fs::write(path, zune_inflate::DeflateEncoder::new(data).encode_zlib()) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(format!("{}", e.to_string()));
                    }
                }
            } else {
                println!("existuje");
            }
        }

        Ok(sha)
    }

    pub fn cat_file(&self, object: String, object_type: ObjectType) -> Result<Vec<u8>, String> {
        let data =
            self.object_read(self.obj_file(object.clone(), object_type.to_string(), None))?;
        let serialized_data = data.serialize();

        Ok(data.data())
    }

    pub fn obj_file(&self, object: String, _fmt: String, _follow: Option<bool>) -> String {
        object
    }

    pub fn hash_obj(
        repo: Option<&GitRepo>,
        path: PathBuf,
        fmt: ObjectType,
    ) -> Result<String, String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        let obj: Box<dyn GitObject> = GitObjectFactory::new(fmt, data);
        GitRepo::object_write(repo, obj)
    }
}
