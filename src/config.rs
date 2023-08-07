use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct PermissionConfig {
    pub mode: u32,
    pub user: String,
    pub group: String,
}

#[derive(Serialize, Deserialize)]
pub struct OmdbConfig {
    pub apikey: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub dir_watch: PathBuf,
    pub show_path: PathBuf,
    pub movie_path: PathBuf,
    pub permissions: PermissionConfig,
    pub omdb: OmdbConfig,
    pub overwrite: bool,
}

impl Config {
    pub fn from_file(file: &Path) -> Result<Config> {
        let config_file = std::fs::File::open(file)?;
        match serde_yaml::from_reader(config_file) {
            Ok(c) => Ok(c),
            Err(e) => bail!("Cannot load config file: {}", e),
        }
    }
}
