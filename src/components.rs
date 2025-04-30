mod alsa;
mod backlight;
mod battery;
mod text;
mod time;

use std::{collections::HashMap, fmt::Debug};

use alsa::{Alsa, AlsaSettings};
use anyhow::Context;
use backlight::{Backlight, BacklightSettings};
use battery::{Battery, BatterySettings};
use delegate::delegate;
use serde::{Deserialize, Deserializer};
use serde_yml::Value;
use smart_default::SmartDefault;
use text::Text;
use time::{Time, TimeSettings};

// ComponentType //////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ComponentType {
    Backlight(Backlight),
    Battery(Battery),
    Time(Time),
    Alsa(Alsa),
    Text(Text),
}

pub trait Component: Debug {
    fn name(&self) -> String;
    fn update(&mut self) -> anyhow::Result<()>;
    fn get_format_str(&self) -> anyhow::Result<String>;
    fn format(&self) -> anyhow::Result<String>;
}

impl Component for ComponentType {
    delegate! {
        to match self {
            Self::Backlight(c) => c,
            Self::Battery(c) => c,
            Self::Time(c) => c,
            Self::Alsa(c) => c,
            Self::Text(c) => c,
        } {
            fn name(&self) -> String;
            fn update(&mut self) -> anyhow::Result<()>;
            fn get_format_str(&self) -> anyhow::Result<String>;
            fn format(&self) -> anyhow::Result<String>;
        }
    }
}

// ComponentVec ///////////////////////////////////////////////////////////////

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

        // flatten into a vec of tuples
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

macro_rules! de_settings_and_instantiate_component {
    ($component_type:ident, $component_settings:ident, $value:tt) => {{
        let settings: $component_settings =
            serde_yml::from_value($value.clone()).context("failed to parse {name} config")?;
        Ok(ComponentType::$component_type($component_type {
            settings,
            ..Default::default()
        }))
    }};
}

fn component_create(name: &str, value: &Value) -> Result<ComponentType, anyhow::Error> {
    let name = name.to_lowercase();
    match name.as_str() {
        "backlight" => {
            de_settings_and_instantiate_component!(Backlight, BacklightSettings, value)
        }
        "battery" => {
            de_settings_and_instantiate_component!(Battery, BatterySettings, value)
        }
        "time" => {
            de_settings_and_instantiate_component!(Time, TimeSettings, value)
        }
        "alsa" => {
            de_settings_and_instantiate_component!(Alsa, AlsaSettings, value)
        }
        "text" => Ok(ComponentType::Text(
            serde_yml::from_value::<Text>(value.clone())
                .context("failed to parse {name} config")?)),
        _ => anyhow::bail!("unknown component: {}", name),
    }
}
