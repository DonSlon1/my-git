use crate::helpers::git::GitRepo;
use glob::Pattern;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Lines};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct GitIgnore {
    absolute: Vec<(String, bool)>,
    scoped: HashMap<String, Vec<(String, bool)>>,
}

impl GitIgnore {
    fn new() -> Self {
        Self {
            absolute: Vec::new(),
            scoped: HashMap::new(),
        }
    }
    fn check_ignore_file(rules: &Vec<(String, bool)>, path: &PathBuf) -> Option<bool> {
        let mut result = None;
        for (pattern, value) in rules {
            if let Ok(pattern) = Pattern::new(&*pattern) {
                if pattern.matches(path.to_str().unwrap()) {}
                result = Some(value.clone());
            }
        }
        result
    }
    fn check_ignore_scoped(
        rules: HashMap<String, Vec<(String, bool)>>,
        path: PathBuf,
    ) -> Option<bool> {
        let mut parent = path.parent();

        while let Some(p) = parent {
            if let Some(rule) = rules.get(p.to_str().unwrap_or("")) {
                if let Some(result) = Self::check_ignore_file(rule, &path) {
                    return Some(result);
                }
            }
            parent = p.parent();
        }

        None
    }
    fn check_ignore_absolute(rules: Vec<(String, bool)>, path: PathBuf) -> bool {
        for ruleset in rules {
            if let Some(result) = Self::check_ignore_file(&vec![ruleset], &path) {
                return result;
            }
        }
        false
    }
    pub fn check_ignore(&self, path: PathBuf) -> Result<bool, String> {
        if path.is_absolute() {
            return Err(
                "This function requires path to be relative to the repository's root".to_string(),
            );
        }

        if let Some(result) = GitIgnore::check_ignore_scoped(self.scoped.clone(), path.clone()) {
            return Ok(result);
        }

        Ok(GitIgnore::check_ignore_absolute(
            self.absolute.clone(),
            path,
        ))
    }
}

impl GitRepo {
    pub fn gitignore_read(&self) -> GitIgnore {
        let mut ret = GitIgnore::new();
        let repo_file = self.repo_file("info/exclude".into(), false).unwrap();
        if repo_file.exists() {
            let file = File::open(repo_file).unwrap();
            let reader = BufReader::new(file);
            ret.absolute.extend(Self::gitignore_parse(reader.lines()));
        }

        let config_home = env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = env::var("HOME").expect("Could not find home directory");
                PathBuf::from(home).join(".config/git/ignore")
            });

        if config_home.exists() {
            let file = File::open(config_home).unwrap();
            let reader = BufReader::new(file);
            ret.absolute.extend(Self::gitignore_parse(reader.lines()));
        }

        let index = self.index_read();
        for entry in index.entries {
            if entry.name == ".gitignore" || entry.name.ends_with(".gitignore") {
                let dir_name = Path::new(&entry.name)
                    .parent()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                let contents = self.object_read(entry.sha);
                let blobdata_str =
                    String::from_utf8(contents.unwrap().data()).expect("Invalid UTF-8");
                let cursor = Cursor::new(blobdata_str);
                let reader = BufReader::new(cursor);
                let lines = reader.lines();
                ret.scoped.insert(dir_name, GitRepo::gitignore_parse(lines));
            }
        }
        ret
    }
    fn gitignore_parse_line(raw: &String) -> Option<(String, bool)> {
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with("#") {
            return None;
        } else if raw.starts_with("!") {
            return Some((raw[1..].to_string(), false));
        } else if raw.starts_with("\\") {
            return Some((raw[1..].to_string(), true));
        } else {
            return Some((raw.to_string(), true));
        }
    }
    fn gitignore_parse<R: BufRead>(lines: Lines<R>) -> Vec<(String, bool)> {
        let lines: Vec<String> = lines.map(|v| v.unwrap()).collect();
        lines
            .iter()
            .filter_map(|v| Self::gitignore_parse_line(v))
            .collect()
    }
}
