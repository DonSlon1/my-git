use crate::helpers::git::GitRepo;
use sha1::digest::generic_array::arr;
use std::io::Read;

#[derive(Debug)]
pub struct GitIndexEntry {
    pub(crate) ctime: (u32, u32),
    pub(crate) mtime: (u32, u32),
    pub(crate) dev: u32,
    pub(crate) ino: u32,
    pub(crate) mode_type: u16,
    pub(crate) mode_perms: u16,
    pub(crate) uid: u32,
    pub(crate) gid: u32,
    fsize: u32,
    pub(crate) sha: String,
    pub(crate) flag_assume_valid: bool,
    pub(crate) flag_stage: u16,
    pub(crate) name: String,
}

impl GitIndexEntry {
    fn new(
        ctime: (u32, u32),
        mtime: (u32, u32),
        dev: u32,
        ino: u32,
        mode_type: u16,
        mode_perms: u16,
        uid: u32,
        gid: u32,
        fsize: u32,
        sha: String,
        flag_assume_valid: bool,
        flag_stage: u16,
        name: String,
    ) -> Self {
        Self {
            ctime,
            mtime,
            dev,
            ino,
            mode_type,
            mode_perms,
            uid,
            gid,
            fsize,
            sha,
            flag_assume_valid,
            flag_stage,
            name,
        }
    }
}

#[derive(Debug)]
pub struct GitIndex {
    pub(crate) version: Option<u32>,
    pub(crate) entries: Vec<GitIndexEntry>,
}

impl GitIndex {
    fn new(version: Option<u32>, entries: Vec<GitIndexEntry>) -> Self {
        Self { version, entries }
    }
}

impl GitRepo {
    pub fn index_read(&self) -> GitIndex {
        let index_file_path = self.repo_file("index".into(), false).unwrap();
        if !index_file_path.exists() {
            return GitIndex::new(None, Vec::new());
        }
        let raw = std::fs::read(index_file_path).unwrap();
        let header = &raw[..12];
        let signature = &header[..4];
        let version = u32::from_be_bytes(header[4..8].try_into().unwrap());
        if version != 2 {
            panic!("mygit supports only index file version 2");
        }
        let count = u32::from_be_bytes(header[8..12].try_into().unwrap());
        let content = &raw[12..];
        let mut entries: Vec<GitIndexEntry> = Vec::new();
        let mut idx = 0;
        for i in 0..count {
            let ctime_s = u32::from_be_bytes(content[idx..idx + 4].try_into().unwrap());
            let time_ns = u32::from_be_bytes(content[idx + 4..idx + 8].try_into().unwrap());
            let mtime_s = u32::from_be_bytes(content[idx + 8..idx + 12].try_into().unwrap());
            let ctime_ns = u32::from_be_bytes(content[idx + 4..idx + 8].try_into().unwrap());
            let mtime_ns = u32::from_be_bytes(content[idx + 12..idx + 16].try_into().unwrap());
            let dev = u32::from_be_bytes(content[idx + 16..idx + 20].try_into().unwrap());
            let ino = u32::from_be_bytes(content[idx + 20..idx + 24].try_into().unwrap());
            let unused = u16::from_be_bytes(content[idx + 24..idx + 26].try_into().unwrap());
            //assert_eq!(unused, 0);
            let mode = u16::from_be_bytes(content[idx + 26..idx + 28].try_into().unwrap());
            let mode_type = mode >> 12;
            //assert!([0b1000, 0b1010, 0b1110].contains(&mode_type));
            let mode_perms = mode & 0b0000000111111111;
            let uid = u32::from_be_bytes(content[idx + 28..idx + 32].try_into().unwrap());
            let gid = u32::from_be_bytes(content[idx + 32..idx + 36].try_into().unwrap());
            let fsize = u32::from_be_bytes(content[idx + 36..idx + 40].try_into().unwrap());
            let sha = hex::encode(&content[idx + 40..idx + 60]);
            let flags = u16::from_be_bytes(content[idx + 60..idx + 62].try_into().unwrap());
            let flag_assume_valid = (flags & 0b1000000000000000) != 0;
            let flag_extended = (flags & 0b0100000000000000) != 0;
            //assert!(!flag_extended);
            let flag_stage = flags & 0b0011000000000000;
            let name_length = flags & 0b0000111111111111;

            // We've read 62 bytes so far.
            idx += 62;
            let name_length = flags & 0b0000111111111111;
            let raw_name: &[u8];
            if name_length < 0xFFF {
                assert_eq!(content[idx + name_length as usize], 0x00);
                raw_name = &content[idx..idx + name_length as usize];
                idx += name_length as usize + 1;
            } else {
                println!("Notice: Name is 0x{:X} bytes long.", name_length);
                let null_idx = content[idx + 0xFFF..]
                    .iter()
                    .position(|&b| b == 0x00)
                    .unwrap()
                    + idx
                    + 0xFFF;
                raw_name = &content[idx..null_idx];
                idx = null_idx + 1;
            }

            let name = std::str::from_utf8(raw_name).expect("Invalid UTF-8 sequence");

            idx = 8 * ((idx + 7) / 8);

            entries.push(GitIndexEntry {
                ctime: (ctime_s, ctime_ns),
                mtime: (mtime_s, mtime_ns),
                dev,
                ino,
                mode_type,
                mode_perms,
                uid,
                gid,
                fsize,
                sha,
                flag_assume_valid,
                flag_stage,
                name: name.to_string(),
            });
        }

        GitIndex::new(Some(count), entries)
    }
}
