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

// a method for Config that parses a component and returns a new instance of it
macro_rules! parse_components {
    ($(($name:expr, $settings_type:ident, $component_type:ident)),*) => {
        fn parse_component(key: &String, value: &Value) -> Result<Box<dyn Component>, Box<dyn Error>> {
            match key.to_lowercase().as_str() {
                $(
                    $name => {
                        let settings: $settings_type = serde_json::from_value(value.clone())
                            .unwrap_or_else(|_| panic!("failed to parse {} config", key));
                        let component = $component_type {
                            settings,
                            ..Default::default()
                        };
                        Ok(Box::new(component))
                    }
                )*,
                _ => Err(format!("can't parse unknown component {}", key).into()),
            }
        }
    };
}

// TODO: use RON instead of json

impl Config {
    pub fn new(args: &Args) -> Result<Self, Box<dyn Error>> {
        let path: PathBuf = args.config_path.clone();
        let contents: String = Self::read_json(&path)?;
        let deserialized: HashMap<String, Value> = Self::deserialize_json(&contents)?;
        let config: Config = Self::parse_config(deserialized)?;
        Ok(config)
    }

    // read config file and return String
    fn read_json(contents: &PathBuf) -> Result<String, Box<dyn Error>> {
        Ok(fs::read_to_string(contents).map_err(|_| "failed to read config file")?)
    }

    // deserialize String json into HashMap
    fn deserialize_json(contents: &str) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        Ok(serde_json::from_str(contents)?)
    }

    // parse HashMap and return generated Config struct
    fn parse_config(hashmap: HashMap<String, Value>) -> Result<Config, Box<dyn Error>> {
        hashmap.into_iter().try_fold(
            Self::default(),
            |acc_config, (category, body)| match category.to_lowercase().as_str() {
                "components" => {
                    let components = body
                        .as_object()
                        .ok_or_else(|| format!("could not parse category: {}", category))?
                        .iter()
                        .map(|(component_name, settings)| {
                            Config::parse_component(component_name, settings).map_err(|_| {
                                format!(
                                    "could not parse component {}: {:?}",
                                    component_name, settings
                                )
                            })
                        })
                        .collect::<Result<Vec<Box<dyn Component>>, _>>()?;

                    Ok(Self {
                        components,
                        ..acc_config
                    })
                }
                "settings" => {
                    let settings = serde_json::from_value(body)
                        .map_err(|_| format!("could not parse category {}", category))?;

                    Ok(Self {
                        settings,
                        ..acc_config
                    })
                }
                x => Err(format!("unknown setting category: {}", x).into()),
            },
        )
    }

    // parse a component and return a new instance of it
    parse_components!(
        ("alsa", AlsaSettings, Alsa),
        ("backlight", BacklightSettings, Backlight),
        ("battery", BatterySettings, Battery),
        ("time", TimeSettings, Time)
    );
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn config_file() {}
// }
