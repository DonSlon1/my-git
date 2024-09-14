use std::fmt::format;
use std::ops::Add;
use std::path::PathBuf;

#[derive(Debug,Clone)]
pub struct GitTreeLeaf{
    pub sha: String,
    pub path: PathBuf,
    pub mode: String 
}

impl GitTreeLeaf {
    pub fn new(sha: String, path: PathBuf, mode: String) -> Self {
        GitTreeLeaf {
            sha,
            path,
            mode,
        }
    }

    pub fn sort_keys(self) -> String {
        if self.mode.starts_with("10") {
            self.path.into_os_string().into_string().unwrap()
        } else {
            self.path.into_os_string().into_string().unwrap().add("/")
        }
    }
    
    pub fn new_from_raw(raw: &[u8],start: Option<usize>) -> Option<(usize,Self)> {
        let start = start.unwrap_or(0);
        let mode_position = match raw.iter().skip(start).position(|&v| v == b' ').map(|pos| pos + start) {
            None => {
                return None;
            }
            Some(v) => v,
        };
        let mode = raw[start..mode_position].to_vec();
        let mode = String::from_utf8(mode.to_vec()).ok()?;
        let mode = format!("{:06}", mode.parse::<u32>().ok()?);
        
        
        let null_terminator =  match raw.iter().skip(mode_position).position(|&v| v == b'\x00').map(|pos| pos + mode_position) {
            None => {
                return None;
            }
            Some(v) => v,
        }; 
        let path = raw[mode_position+1..null_terminator].to_vec();
        let sha_bytes = &raw[null_terminator + 1 ..null_terminator + 21];
        let sha: String = sha_bytes.iter().map(|byte| format!("{:02x}", byte)).collect();

        // Format the integer as a zero-padded hexadecimal string
        let path = String::from_utf8(path).unwrap();
        let git_tree_leaf = GitTreeLeaf::new(sha,path.into(),mode);
        Some((null_terminator+21,git_tree_leaf))
    }
}