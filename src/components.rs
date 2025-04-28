mod backlight;
mod battery;

use std::{collections::HashMap, fmt::Debug};

use anyhow::Context;
use backlight::{Backlight, BacklightSettings};
use battery::{Battery, BatterySettings};
use serde::{Deserialize, Deserializer};
use serde_yml::Value;
use smart_default::SmartDefault;

// ComponentType

#[derive(Debug)]
pub enum ComponentType {
    Backlight(Backlight),
    Battery(Battery),
    // Time(Time),
}

impl Component for ComponentType {
    fn name(&self) -> String {
        match self {
            ComponentType::Backlight(c) => c.name(),
            ComponentType::Battery(c) => c.name(),
        }
    }

    fn update(&mut self) -> anyhow::Result<()> {
        match self {
            ComponentType::Backlight(c) => c.update(),
            ComponentType::Battery(c) => c.update(),
        }
    }

    fn get_format_str(&self) -> anyhow::Result<String> {
        match self {
            ComponentType::Backlight(c) => c.get_format_str(),
            ComponentType::Battery(c) => c.get_format_str(),
        }
    }

    fn format(&self) -> anyhow::Result<String> {
        match self {
            ComponentType::Backlight(c) => c.format(),
            ComponentType::Battery(c) => c.format(),
        }
    }
}

// ComponentVec

#[derive(SmartDefault, Debug)]
pub struct ComponentVec {
    #[default(Vec::new())]
    pub vec: Vec<ComponentType>,
}

impl<'de> Deserialize<'de> for ComponentVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into a vec of length-one hashmaps first
        let components_raw: Vec<HashMap<String, Value>> = Vec::deserialize(deserializer)?;

        let components_flattened: Vec<(String, Value)> = components_raw
            .into_iter()
            .flat_map(|map| map.into_iter())
            .collect();

        // Parse each component
        let components_new: Vec<ComponentType> = components_flattened
            .iter()
            .map(|(component_name, settings)| {
                component_create(component_name, settings).map_err(|e| {
                    serde::de::Error::custom(format!(
                        "could not parse component {}: {}",
                        component_name, e
                    ))
                })
            })
            .collect::<Result<_, D::Error>>()?;

        Ok(ComponentVec {
            vec: components_new,
        })
    }
}

fn component_create(name: &str, value: &Value) -> Result<ComponentType, anyhow::Error> {
    let name = name.to_lowercase();
    // TODO: search through all componentTypes and match based on name
    match name.as_str() {
        "backlight" => {
            let settings: BacklightSettings =
                serde_yml::from_value(value.clone()).context("failed to parse {name} config")?;
            let res: Backlight = Backlight {
                settings,
                ..Backlight::default()
            };
            Ok(ComponentType::Backlight(res))
        }
        "battery" => {
            let settings: BatterySettings =
                serde_yml::from_value(value.clone()).context("failed to parse {name} config")?;
            let res: Battery = Battery {
                settings,
                ..Battery::default()
            };
            Ok(ComponentType::Battery(res))
        }
        // "time" => {
        //     let settings: TimeSettings
        // },
        _ => {
            anyhow::bail!("unknown component: {}", name);
        }
    }
}

pub trait Component: Debug {
    fn name(&self) -> String;
    fn update(&mut self) -> anyhow::Result<()>;
    fn get_format_str(&self) -> anyhow::Result<String>;
    fn format(&self) -> anyhow::Result<String>;
}
