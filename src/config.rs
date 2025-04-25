use anyhow::Context;
use serde::Deserialize;
use smart_default::SmartDefault;
use std::{fs, path::PathBuf};

use crate::Args;
use crate::components::ComponentList;

#[derive(SmartDefault, Debug, Deserialize)]
pub struct Config {
    pub settings: Settings,
    pub components: ComponentList,
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
        let contents: String = Self::read_file(&path)?;
        let config: Config = serde_yml::from_str(&contents)?;
        Ok(config)
    }

    // read config file and return String
    fn read_file(contents: &PathBuf) -> anyhow::Result<String> {
        fs::read_to_string(contents).with_context(|| "failed to read config file")
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn config_file() {}
// }
