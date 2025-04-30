use std::{env, fs, path::PathBuf};

use anyhow::Context;
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::{args::Args, components::ComponentVec};

pub fn default_config_path() -> PathBuf {
    let config_dir = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|p| p.join(".config"))
                .expect("Cannot find HOME directory")
        });

    config_dir.join("ferristatus").join("config.yml")
}

#[derive(SmartDefault, Debug, Deserialize)]
pub struct Config {
    pub settings: Settings,
    pub components: ComponentVec,
}

#[derive(SmartDefault, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    #[default(100)]
    pub check_interval: u64,

    #[default("|")]
    pub default_separator: String,
}

impl Config {
    pub fn new(args: &Args) -> anyhow::Result<Self> {
        let path: PathBuf = args.config_path.clone();
        let contents: String = read_file(&path)?;
        let config: Config = serde_yml::from_str(&contents)?;
        Ok(config)
    }
}

// read config file and return String
fn read_file(contents: &PathBuf) -> anyhow::Result<String> {
    fs::read_to_string(contents).with_context(|| "failed to read config file")
}
