use std::{fs, io};
use std::fs::File;
use crate::helpers::git::GitRepo;
use sha1::digest::generic_array::arr;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use hex::decode;

#[derive(Debug,Clone)]
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

        GitIndex::new(Some(version), entries)
    }
    fn index_write(&self, index: &GitIndex) -> io::Result<()> {
        let index_file = self.repo_file("index".into(),false).unwrap();
        let mut file = File::create(index_file)?;

        // HEADER
        file.write_all(b"DIRC")?;
        file.write_all(&index.version.unwrap_or(2_u32).to_be_bytes())?;
        file.write_all(&(index.entries.len() as u32).to_be_bytes())?;

        // ENTRIES
        let mut idx = 0;
        for e in &index.entries {
            file.write_all(&e.ctime.0.to_be_bytes())?;
            file.write_all(&e.ctime.1.to_be_bytes())?;
            file.write_all(&e.mtime.0.to_be_bytes())?;
            file.write_all(&e.mtime.1.to_be_bytes())?;
            file.write_all(&e.dev.to_be_bytes())?;
            file.write_all(&e.ino.to_be_bytes())?;

            // Mode
            let mode = (e.mode_type << 12) | e.mode_perms;
            file.write_all(&mode.to_be_bytes())?;

            file.write_all(&e.uid.to_be_bytes())?;
            file.write_all(&e.gid.to_be_bytes())?;
            file.write_all(&e.fsize.to_be_bytes())?;

            // SHA
            let sha_bytes = hex::decode(&e.sha).expect("Invalid SHA");
            file.write_all(&sha_bytes)?;

            let flag_assume_valid = if e.flag_assume_valid { 0x1 << 15 } else { 0 };
            let name_bytes = e.name.as_bytes();
            let bytes_len = name_bytes.len();
            let name_length = if bytes_len >= 0xFFF { 0xFFF } else { bytes_len as u16 };

            // Flags and name length
            let flags = flag_assume_valid | e.flag_stage | name_length;
            file.write_all(&flags.to_be_bytes())?;

            // Name and padding
            file.write_all(name_bytes)?;
            file.write_all(&[0])?;

            idx += 62 + bytes_len + 1;
            if idx % 8 != 0 {
                let pad = 8 - (idx % 8);
                file.write_all(&vec![0; pad])?;
                idx += pad;
            }
        }

        Ok(())
    }
    
    /// Removes files from the Git index and optionally deletes them from the filesystem.
    ///
    /// # Arguments
    ///
    /// * `paths` - A slice of paths to remove.
    /// * `delete` - Whether to delete the files from the filesystem.
    /// * `skip_missing` - Whether to skip paths not found in the index.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` with an error message if any operation fails.
    pub fn rm<P: AsRef<Path>>(
        &self,
        paths: &Vec<P>,
        delete: bool,
        skip_missing: bool,
    ) -> Result<(), String> {
        // Step 1: Read the current index
        let mut index = self.index_read();

        // Step 2: Define the worktree path as a PathBuf
        let worktree = fs::canonicalize(Path::new(&self.work_dir).to_path_buf()).map_err(|e| {
                format!(
                    "Error: Cannot canonicalize worktree path {:?}: {}",
                    self.work_dir, e
                )
            })?;

        // Step 3: Convert input paths to absolute paths and validate
        let mut abspaths: Vec<PathBuf> = Vec::new();
        for path in paths {
            let abspath = if path.as_ref().is_absolute() {
                PathBuf::from(path.as_ref())
            } else {
                match fs::canonicalize(path.as_ref()) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!(
                            "Error: Cannot canonicalize path {:?}: {}",
                            path.as_ref(),
                            e
                        );
                        return Err(format!(
                            "Cannot canonicalize path {:?}: {}",
                            path.as_ref(),
                            e
                        ));
                    }
                }
            };

            // Check if the absolute path starts with the worktree
            if !abspath.starts_with(&worktree) {
                eprintln!("workdir {:?}",worktree);
                eprintln!(
                    "Error: Cannot remove paths outside of worktree: {:?}",
                    path.as_ref()
                );
                return Err(format!(
                    "Cannot remove paths outside of worktree: {:?}",
                    path.as_ref()
                ));
            }

            abspaths.push(abspath);
        }

        // Step 4: Iterate through index entries to identify which to remove
        let mut kept_entries: Vec<GitIndexEntry> = Vec::new();
        let mut remove_paths: Vec<PathBuf> = Vec::new();

        for entry in &index.entries {
            let full_path = worktree.join(&entry.name);

            if abspaths.contains(&full_path) {
                remove_paths.push(full_path.clone());
            } else {
                kept_entries.push(entry.clone());
            }
        }

        // Step 5: Check for paths that were not found in the index
        if !abspaths.is_empty() {
            let mut not_found: Vec<String> = Vec::new();
            for path in &abspaths {
                if !remove_paths.contains(path) {
                    not_found.push(path.to_string_lossy().into_owned());
                }
            }

            if !not_found.is_empty() && !skip_missing {
                eprintln!(
                    "Error: Cannot remove paths not in the index: {:?}",
                    not_found
                );
                return Err(format!(
                    "Cannot remove paths not in the index: {:?}",
                    not_found
                ));
            }
        }

        // Step 6: Optionally delete the files from the filesystem
        if delete {
            for path in &remove_paths {
                if path.is_file() {
                    if let Err(e) = fs::remove_file(path) {
                        eprintln!("Error: Failed to delete file {:?}: {}", path, e);
                        return Err(format!("Failed to delete file {:?}: {}", path, e));
                    }
                } else if path.is_dir() {
                    if let Err(e) = fs::remove_dir_all(path) {
                        eprintln!("Error: Failed to delete directory {:?}: {}", path, e);
                        return Err(format!("Failed to delete directory {:?}: {}", path, e));
                    }
                } else {
                    eprintln!("Warning: Path {:?} is neither file nor directory.", path);
                }
            }
        }

        // Step 7: Update the index with kept entries
        index.entries = kept_entries;

        // Step 8: Write the updated index back
        match self.index_write(&index) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Error: Failed to write index: {}", e);
                Err(format!("Failed to write index: {}", e))
            }
        }
    }
}
