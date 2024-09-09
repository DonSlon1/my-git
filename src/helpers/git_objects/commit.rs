use crate::helpers::git_objects::git_object::GitObject;

pub struct GitCommit<'a> {
    fmt: &'a [u8],
    data: Vec<u8>,
}

impl<'a> GitCommit<'a> {
    pub fn new(data:Vec<u8>) -> Self {
        GitCommit { fmt: b"commit", data }
    }
}

impl GitObject for GitCommit<'_> {
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
