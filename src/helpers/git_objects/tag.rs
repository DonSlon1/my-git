use std::any::Any;
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

    fn as_ref(&self) -> Box<dyn Any> {
        todo!()
    }
}
