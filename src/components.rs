use crate::utils::safe_strfmt;
use serde::{de, Deserialize, Deserializer};
use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
};

mod alsa;
mod backlight;
mod battery;
mod component_list;
mod time;
mod utils;

pub use component_list::*;

pub trait Component: Debug + Display
{
    fn name(&self) -> String;
    fn update(&mut self) -> anyhow::Result<()>;
    fn get_format_string(&self) -> String;
    fn eval_strfmt(&self, format_str: &str) -> anyhow::Result<String>;
    fn new_from_settings<T>(settings: T) -> Self
    where
        T: ComponentSettings + Into<Self>, Self: Sized
    {
        settings.into()
    }
}

pub trait ComponentSettings: Debug + for<'a> Deserialize<'a> {}

pub trait ComponentFormatSettings: Debug {
    /// Custom deserializer that does strfmt preprocessing with vars
    fn de_strfmt<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized + Deserialize<'de>,
    {
        // Deserialize raw settings
        let format = Self::deserialize(deserializer)?;

        // copy from vars field
        let vars = format.get_vars().clone();

        // Apply formatting transformations
        let formatted = Self::de_strfmt_formatting(&format, vars)
            .map_err(de::Error::custom)?;

        Ok(formatted)
    }

    fn get_vars(&self) -> &HashMap<String, String>;

    fn get_levels(&self) -> Option<&Vec<(i32, String)>>;

    fn de_strfmt_formatting<'de>(
        orig: &Self,
        vars: HashMap<String, String>,
    ) -> Result<Self, String>
    where
        Self: Sized;

    fn safe_strfmt_levels(
        &self,
        vars: &HashMap<String, String>,
    ) -> anyhow::Result<Vec<(i32, String)>, Box<dyn Error>> {
        let mut levels: Vec<(i32, String)> =
            self.get_levels()
                .ok_or("no levels found".to_string())?
                .clone()
                .into_iter()
                .map(|(k, v)| (k, safe_strfmt(&v, vars)))
                .collect();
        levels.sort_by_key(|(k, _)| *k);
        Ok(levels)
    }
}

pub trait ComponentState: Debug {}
