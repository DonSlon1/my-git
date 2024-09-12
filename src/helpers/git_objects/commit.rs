use std::hash::RandomState;
use ordermap::OrderMap;
use crate::helpers::git_objects::git_object::GitObject;
use crate::helpers::kvlm::{kvlm_parse, kvlm_serilize};

pub struct GitCommit<'a> {
    fmt: &'a [u8],
    data: Vec<u8>,
    kvlm: OrderMap<Vec<u8>,Vec<Vec<u8>>,RandomState>
}

impl<'a> GitCommit<'a> {
    pub fn new(data: Vec<u8>) -> Self {
        let borrowed_kvlm = kvlm_parse(&data, None, None);

        // Convert borrowed OrderMap to owned OrderMap
        let kvlm: OrderMap<Vec<u8>, Vec<Vec<u8>>, RandomState> = borrowed_kvlm.into_iter()
            .map(|(k, v)| (k.to_vec(), v))
            .collect();
        GitCommit {
            fmt: b"commit",
            data,
            kvlm,
        }
    }
}

impl GitObject for GitCommit<'_> {
    fn serialize(&self) -> Vec<u8> {
        kvlm_serilize(self.kvlm.clone())
    }

    fn deserialize(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn fmt(&self) -> &[u8] {
        self.fmt
    }
}
