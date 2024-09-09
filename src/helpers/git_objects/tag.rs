use crate::helpers::git_objects::git_object::GitObject;

pub struct GitTag<'a> {
    fmt: &'a [u8],
    data: Vec<u8>,
}

impl<'a> GitTag<'a> {
    pub fn new(data:Vec<u8>) -> Self {
        GitTag { fmt: b"tag", data }
    }
}

impl GitObject for GitTag<'_> {
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
