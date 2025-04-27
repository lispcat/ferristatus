use super::alsa::{Alsa, AlsaSettings};
use super::backlight::{Backlight, BacklightSettings};
use super::battery::{Battery, BatterySettings};
use super::time::{Time, TimeSettings};
use super::Component;
use serde::{Deserialize, Deserializer};
use serde_yml::Value;
use smart_default::SmartDefault;
use std::collections::HashMap;
use std::error::Error;

#[derive(SmartDefault, Debug)]
pub struct ComponentList {
    #[default(Vec::new())]
    pub list: Vec<Box<dyn Component>>,
}

macro_rules! component_creator {
    ( $(( $name:expr, $settings_type:ident, $component_type:ident )),* ) => {
        fn component_create(key: &String, value: &Value) -> Result<Box<dyn Component>, Box<dyn Error>> {
            match key.to_lowercase().as_str() {
                $(
                    $name => {
                        let settings: $settings_type = serde_yml::from_value(value.clone())
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

component_creator!(
    ("alsa", AlsaSettings, Alsa),
    ("backlight", BacklightSettings, Backlight),
    ("battery", BatterySettings, Battery),
    ("time", TimeSettings, Time)
);

impl<'de> Deserialize<'de> for ComponentList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into an IndexMap first
        let components_vec: Vec<HashMap<String, Value>> = Vec::deserialize(deserializer)?;

        let components_flattened: Vec<(String, Value)> = components_vec
            .into_iter()
            .flat_map(|map| map.into_iter())
            .collect();

        // Parse each component
        let component_list = components_flattened
            .iter()
            .map(|(component_name, settings)| {
                component_create(component_name, settings).map_err(|e| {
                    serde::de::Error::custom(format!(
                        "could not parse component {}: {}",
                        component_name, e
                    ))
                })
            })
            .collect::<Result<Vec<Box<dyn Component>>, D::Error>>()?;

        Ok(ComponentList {
            list: component_list,
        })
    }
}
