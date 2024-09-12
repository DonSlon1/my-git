use std::any::Any;
use crate::helpers::git_objects::git_object::GitObject;

#[derive(Debug, Clone)]
pub struct GitBlob {
    fmt: Vec<u8>,
    data: Vec<u8>,
}

impl GitBlob {
    pub fn new(data:Vec<u8>) -> Self {
        GitBlob { fmt: b"blob".to_vec(), data }
    }
}

impl GitObject for GitBlob {
    fn serialize(&self) -> String {
        self.data.iter()
            .filter(|&byte| {
                byte.is_ascii() && (byte.is_ascii_graphic() || byte.is_ascii_whitespace())
            })
            .map(|&byte| {
                byte as char
            }).collect::<String>()
    }
    fn deserialize(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn format(&self) -> Vec<u8> {
        self.fmt.clone()
    }

    fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}
