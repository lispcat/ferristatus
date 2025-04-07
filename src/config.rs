use anyhow::Result;
use serde::{Deserialize, Deserializer};
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

#[derive(SmartDefault, Debug, Deserialize)]
pub struct Config {
    pub settings: Settings,

    pub components: Components,
}

#[derive(SmartDefault, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    #[default(100)]
    pub check_interval: u64,

    #[default("|")]
    pub default_separator: String,
}

#[derive(SmartDefault, Debug)]
#[default(Vec::new())]
pub struct Components(pub Vec<Box<dyn Component>>);

macro_rules! component_parser {
    ($(($name:expr, $settings_type:ident, $component_type:ident)),*) => {
        fn component_parser(key: &String, value: &Value) -> Result<Box<dyn Component>, Box<dyn Error>> {
            match key.to_lowercase().as_str() {
                $(
                    $name => {
                        let settings: $settings_type = serde_json::from_value(value.clone())
                            .map_err(|_| format!("failed to parse {} config", key))?;
                        Ok(Box::new($component_type {
                            settings,
                            ..Default::default()
                        }))
                    }
                )*,
                _ => Err(format!("can't parse unknown component {}", key).into()),
            }
        }
    };
}

impl Components {
    component_parser!(
        ("alsa", AlsaSettings, Alsa),
        ("backlight", BacklightSettings, Backlight),
        ("battery", BatterySettings, Battery),
        ("time", TimeSettings, Time)
    );
}

/// make Components deserializable
impl<'de> Deserialize<'de> for Components {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into a HashMap first
        let components_map: HashMap<String, Value> = HashMap::deserialize(deserializer)?;

        // Parse each component
        let component_vec = components_map
            .iter()
            .map(|(component_name, settings)| {
                Components::component_parser(component_name, settings).map_err(|e| {
                    serde::de::Error::custom(format!(
                        "could not parse component {}: {}",
                        component_name, e
                    ))
                })
            })
            .collect::<Result<Vec<Box<dyn Component>>, D::Error>>()?;

        Ok(Components(component_vec))
    }
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
