use configparser::ini;
use configparser::ini::{Ini, WriteOptions};
use std::path::PathBuf;

pub fn get_default_conf() -> Ini {
    let mut config = Ini::new();

    config.set("core", "repositoryformatversion", Some("0".to_string()));
    config.set("core", "filemode", Some("true".to_string()));
    config.set("core", "bare", Some("false".to_string()));
    config.set("core", "logallrefupdates", Some("true".to_string()));

    config
}

pub fn write_conf(path: PathBuf, config: Ini) -> std::io::Result<()> {
    config.pretty_write(path, &get_default_write_options())
}

fn get_default_write_options() -> WriteOptions {
    WriteOptions::new_with_params(true, 1, 4)
}

pub fn read_conf(path: PathBuf) -> Result<Ini, String> {
    if !path.is_file() {
        return Err("config is not a file".to_string());
    }
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => return Err(e.to_string()),
    };

    let mut config = Ini::new();
    config.read(content)?;

    Ok(config)
}
