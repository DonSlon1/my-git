use crate::helpers::git_objects::git_object::GitObject;

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
    fn serialize(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn deserialize(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn fmt(&self) -> &[u8] {
        self.fmt
    }
}
