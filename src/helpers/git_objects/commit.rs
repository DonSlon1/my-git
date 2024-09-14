use std::any::Any;
use std::hash::RandomState;
use ordermap::OrderMap;
use crate::helpers::git_objects::git_object::GitObject;
use crate::helpers::kvlm::{kvlm_parse, kvlm_serialize};

#[derive(Debug, Clone)]
pub struct GitCommit {
    fmt: Vec<u8>,
    data: Vec<u8>,
    pub kvlm: OrderMap<Vec<u8>,Vec<Vec<u8>>,RandomState>
}

impl GitCommit {
    pub fn new(data: Vec<u8>) -> Self {
        let borrowed_kvlm = kvlm_parse(&data, None, None);

        // Convert borrowed OrderMap to owned OrderMap
        let kvlm: OrderMap<Vec<u8>, Vec<Vec<u8>>, RandomState> = borrowed_kvlm.into_iter()
            .map(|(k, v)| (k.to_vec(), v))
            .collect();
        GitCommit {
            fmt: b"commit".to_vec(),
            data,
            kvlm,
        }
    }
}

impl GitObject for GitCommit {
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
