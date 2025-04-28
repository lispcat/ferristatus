mod backlight;

use std::{collections::HashMap, fmt::Debug, fmt::Display};

use anyhow::Context;
use backlight::{Backlight, BacklightSettings};
use serde::{Deserialize, Deserializer};
use serde_yml::Value;
use smart_default::SmartDefault;

// ComponentType

#[derive(Debug)]
pub enum ComponentType {
    Backlight(Backlight),
    // Time(Time),
}

impl Component for ComponentType {
    fn name(&self) -> String {
        match self {
            ComponentType::Backlight(b) => b.name(),
        }
    }

    fn update(&mut self) -> anyhow::Result<()> {
        match self {
            ComponentType::Backlight(b) => b.update(),
        }
    }

    fn get_format_str(&self) -> anyhow::Result<String> {
        match self {
            ComponentType::Backlight(b) => b.get_format_str(),
        }
    }

    fn format(&self) -> anyhow::Result<String> {
        match self {
            ComponentType::Backlight(b) => b.format(),
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
