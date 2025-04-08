use super::alsa::{Alsa, AlsaSettings};
use super::backlight::{Backlight, BacklightSettings};
use super::battery::{Battery, BatterySettings};
use super::time::{Time, TimeSettings};
use super::Component;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use smart_default::SmartDefault;
use std::{collections::HashMap, error::Error};

#[derive(SmartDefault, Debug)]
#[default(Vec::new())]
pub struct ComponentList {
    pub list: Vec<Box<dyn Component>>,
}

macro_rules! component_parser {
    ( $(( $name:expr, $settings_type:ident, $component_type:ident )),* ) => {
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

impl ComponentList {
    component_parser!(
        ("alsa", AlsaSettings, Alsa),
        ("backlight", BacklightSettings, Backlight),
        ("battery", BatterySettings, Battery),
        ("time", TimeSettings, Time)
    );
}

impl<'de> Deserialize<'de> for ComponentList {
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
                ComponentList::component_parser(component_name, settings).map_err(|e| {
                    serde::de::Error::custom(format!(
                        "could not parse component {}: {}",
                        component_name, e
                    ))
                })
            })
            .collect::<Result<Vec<Box<dyn Component>>, D::Error>>()?;

        Ok(ComponentList {
            list: component_vec,
        })
    }
}
