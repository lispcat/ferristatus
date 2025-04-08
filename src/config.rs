use anyhow::Result;
use serde::Deserialize;
use smart_default::SmartDefault;
use std::{error::Error, fs, path::PathBuf};

use crate::{components::component_list::ComponentList, Args};

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
    pub fn new(args: &Args) -> Result<Self, Box<dyn Error>> {
        let path: PathBuf = args.config_path.clone();
        let contents: String = Self::read_json(&path)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    // read config file and return String
    fn read_json(contents: &PathBuf) -> Result<String, Box<dyn Error>> {
        Ok(fs::read_to_string(contents).map_err(|_| "failed to read config file")?)
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn config_file() {}
// }
