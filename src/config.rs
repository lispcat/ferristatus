use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use smart_default::SmartDefault;
use std::{collections::HashMap, error::Error, fs, path::PathBuf};

use crate::{
    components::{
        alsa::{Alsa, AlsaSettings},
        backlight::{Backlight, BacklightSettings},
        battery::{Battery, BatterySettings},
        time::{Time, TimeSettings},
        Component,
    },
    Args,
};

#[derive(SmartDefault, Debug)]
pub struct Config {
    pub settings: Settings,

    #[default(Vec::new())]
    pub components: Vec<Box<dyn Component>>,
}

#[derive(SmartDefault, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    #[default(100)]
    pub check_interval: u64,

    #[default("|")]
    pub default_separator: String,
}

// TODO: use RON instead of json

impl Config {
    pub fn new(args: &Args) -> Result<Self, Box<dyn Error>> {
        let path: PathBuf = args.config_path.clone();
        let contents: String = Self::read_json(&path);
        let deserialized: HashMap<String, Value> = Self::deserialize_json(&contents)?;
        let config: Config = Self::parse_config(deserialized)?;
        Ok(config)
    }

    // read config file and return String
    fn read_json(contents: &PathBuf) -> String {
        fs::read_to_string(contents).expect("failed to read config file")
    }

    // deserialize String json into HashMap
    fn deserialize_json(contents: &str) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        Ok(serde_json::from_str(contents)?)
    }

    // parse HashMap and return generated Config struct
    fn parse_config(hashmap: HashMap<String, Value>) -> Result<Config, Box<dyn Error>> {
        let mut config = Self::default();

        for (category, value) in hashmap {
            let category = category.to_lowercase();
            match category.as_str() {
                "components" => {
                    let list = value
                        .as_object()
                        .unwrap_or_else(|| panic!("could not parse category: {}", category));
                    let mut vec: Vec<Box<dyn Component>> = list
                        .iter()
                        .map(|(c, v)| {
                            Config::parse_component(c, v).unwrap_or_else(|_| {
                                panic!("could not parse component {}: {:?}", c, v)
                            })
                        })
                        .collect();

                    config.components.append(&mut vec);
                }
                "settings" => {
                    config.settings = serde_json::from_value(value.clone())
                        .unwrap_or_else(|_| panic!("could not parse category {}", category));
                }
                x => return Err(format!("unknown setting category: {}", x).into()),
            }
        }

        Ok(config)
    }

    // parse a component and return a new instance of it
    fn parse_component(key: &String, value: &Value) -> Result<Box<dyn Component>, Box<dyn Error>> {
        match key.to_lowercase().as_str() {
            "alsa" => {
                let settings: AlsaSettings = serde_json::from_value(value.clone())
                    .unwrap_or_else(|_| panic!("failed to parse {} config", key));
                let component = Alsa {
                    settings,
                    ..Default::default()
                };
                Ok(Box::new(component))
            }
            "backlight" => {
                let settings: BacklightSettings = serde_json::from_value(value.clone())
                    .unwrap_or_else(|_| panic!("failed to parse {} config", key));
                let component = Backlight {
                    settings,
                    ..Default::default()
                };
                Ok(Box::new(component))
            }
            "battery" => {
                let settings: BatterySettings = serde_json::from_value(value.clone())
                    .unwrap_or_else(|_| panic!("failed to parse {} config", key));
                let component = Battery {
                    settings,
                    ..Default::default()
                };
                Ok(Box::new(component))
            }
            "time" => {
                let settings: TimeSettings = serde_json::from_value(value.clone())
                    .unwrap_or_else(|_| panic!("failed to parse {} config", key));
                let component = Time {
                    settings,
                    ..Default::default()
                };
                Ok(Box::new(component))
            }
            _ => Err(format!("can't parse unknown component {}", key).into()),
        }
    }
}

#[cfg(test)]
mod tests {

    // use super::*;

    // #[test]
    // fn config_file() {
    //     // let path = PathBuf::from("config.json");
    //     // let contents = Config::read_json(&path);
    //     // let deserialized: HashMap<String, Value> = serde_json::from_str(&contents).unwrap();
    //     // let config: Config = Config::parse_config(deserialized).unwrap();

    //     let config = Config::new(PathBuf::from("config.json")).unwrap();

    //     // println!("TEST: {}", config.components[1]);

    //     println!("DEBUG: OUTPUT: {:#?}", config);
    // }
}
