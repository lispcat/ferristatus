use serde::{Deserialize, Deserializer};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use crate::utils::safe_strfmt;

/// A whole component
pub trait Component: Debug + Display {
    fn name(&self) -> String;
    fn update(&mut self) -> anyhow::Result<()>;
    fn get_format_string(&self) -> String;
    fn eval_strfmt(&self, format_str: &str) -> anyhow::Result<String>;
}

/// User-defined component settings
pub trait ComponentSettings: Debug + for<'a> Deserialize<'a> {}

/// Format settings, required for some components
pub trait ComponentFormat: Debug {
    /// Custom deserializer that does strfmt preprocessing with vars
    fn de_strfmt<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized;

    fn get_levels(&self) -> Option<&Vec<(i32, String)>>;

    fn safe_strfmt_levels(&self, vars: &HashMap<String, String>) -> Vec<(i32, String)> {
        let mut levels: Vec<(i32, String)> = self
            .get_levels()
            .unwrap()
            .clone()
            .into_iter()
            .map(|(k, v)| (k, safe_strfmt(&v, vars)))
            .collect();
        levels.sort_by_key(|(k, _)| *k);
        levels
    }
}

/// last-fetched state of the component
pub trait ComponentState: Debug {}
