mod backlight;

use core::fmt;
use std::{collections::HashMap, fmt::{Debug, Display}, time::{Duration, Instant}};

use anyhow::Context;
use backlight::Backlight;
use serde::{Deserialize, Deserializer};
use serde_yml::Value;
use smart_default::SmartDefault;


///////////////////////////////////////////////////////////////////////////////
//                              Component Traits                             //
///////////////////////////////////////////////////////////////////////////////


// trait Component ////////////////////////////////////////////////////////////

pub trait Component: Debug {
    fn name(&self) -> String;
    fn new_from_value(value: &Value) -> anyhow::Result<Self>
    where
        Self: std::marker::Sized;

    fn update_state(&mut self) -> anyhow::Result<()>;
    fn set_cache(&mut self, str: String) -> anyhow::Result<()>;
    fn get_strfmt_template(&self) -> anyhow::Result<Option<&str>>;
    fn apply_strfmt_template(&self, template: &str) -> anyhow::Result<String>;
    fn update(&mut self) -> anyhow::Result<()> {
        self.update_state().context("failed to update state for component")?;

        let template: Option<&str> = self.get_strfmt_template()?;

        let output = match template {
            Some(t) => self.apply_strfmt_template(t)?,
            None => self.default_output()?.to_string(),
        };

        self.set_cache(output)?;

        Ok(())
    }
    fn update_check(&self) -> anyhow::Result<bool> {
        let last_updated: &Instant = match self.get_last_updated()? {
            Some(v) => v,
            None => return Ok(true),
        };
        let interval = Duration::from_millis(
            *self.get_refresh_interval()?
        );
        let elapsed = last_updated.elapsed();
        Ok(elapsed > interval)
    }
    fn update_maybe(&mut self) -> anyhow::Result<bool> {
        match self.update_check()? {
            true => {
                self.update()?;
                Ok(true)
            },
            false => {
                Ok(false)
            }
        }
    }
    fn get_last_updated(&self) -> anyhow::Result<&Option<std::time::Instant>>;
    fn get_cache(&self) -> anyhow::Result<&Option<String>>;
    fn get_refresh_interval(&self) -> anyhow::Result<&u64>;
    fn default_output(&self) -> anyhow::Result<&str>;
}

impl Display for dyn Component {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = self.get_cache()
            .expect("failed to get component cache")
            .as_ref()
            .map_or("N/A(no_cache_to_display)",  |v| v.as_str());
        write!(f, "{}", out)
    }
}

///////////////////////////////////////////////////////////////////////////////
//                                ComponentVec                               //
///////////////////////////////////////////////////////////////////////////////


#[derive(SmartDefault, Debug)]
pub struct ComponentVec {
    #[default(Vec::new())]
    pub vec: Vec<Box<dyn Component>>,
}

macro_rules! create_component_from_name {
    ( $name:expr, $value:expr, $( $module_name:literal => $module_type:ty ),+ $(,)? ) => {
        match $name.to_lowercase().as_str() {
            $(
                $module_name => Ok(Box::new(
                    <$module_type>::new_from_value($value)?
                )),
            )+
            _ => todo!(),
        }
    };
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
        let components_new: Vec<Box<dyn Component>> = components_flattened.iter()
            .map(|(name, value)| -> anyhow::Result<Box<dyn Component>> {
                create_component_from_name!(
                    name, value,
                    "backlight" => Backlight,
                )
            })
            .collect::<Result<_, anyhow::Error>>()
            .map_err(|e| {
                serde::de::Error::custom(
                    format!("could not parse settings for component: {}", e)
                )
            })?;

        Ok(ComponentVec {
            vec: components_new,
        })
    }
}
