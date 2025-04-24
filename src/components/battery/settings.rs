use std::{collections::HashMap, path::PathBuf};

use crate::utils::safe_strfmt;
use serde::{Deserialize, Deserializer};
use smart_default::SmartDefault;

use crate::components::component_utils::{de_vars_as_flat_hashmap, VarPreprocessing};
use crate::components::{ComponentFormat, ComponentSettings};

/// Settings for the Battery component.
#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatterySettings {
    #[default(1000)]
    pub refresh_interval: u32,

    #[default(7)]
    pub signal: u32,

    #[default(PathBuf::from("/sys/class/power_supply/BAT0"))]
    pub path: PathBuf,

    #[serde(deserialize_with = "BatteryFormatSettings::de_strfmt")]
    pub format: BatteryFormatSettings,
}
impl ComponentSettings for BatterySettings {}

#[derive(Debug, SmartDefault, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryFormatSettings {
    #[default(HashMap::from([
        ("def".to_string(), "{percent}% {time_remaining}".to_string())
    ]))]
    #[serde(deserialize_with = "de_vars_as_flat_hashmap")]
    pub vars: HashMap<String, String>,

    #[default(" Full({percent}) ".to_string())]
    pub full: String,

    #[default(" ? {$def} ".to_string())]
    pub not_charging: String,

    #[default("  {$def} ".to_string())]
    pub charging: String,

    #[default(vec![
        (100, " Full({percent}) ".to_string()),
        (99,  "  {$def} ".to_string()),
        (70,  "  {$def} ".to_string()),
        (50,  "  {$def} ".to_string()),
        (30,  "  {$def} ".to_string()),
        (10,  "  {$def} ".to_string()),
    ])]
    pub discharging: Vec<(i32, String)>,

    #[default(" ? {$def} ")]
    pub default: String,
}
impl ComponentFormat for BatteryFormatSettings {}
impl VarPreprocessing for BatteryFormatSettings {
    fn get_levels(&self) -> &Vec<(i32, String)> {
        &self.discharging
    }
}

impl BatteryFormatSettings {
    pub fn de_strfmt<'de, D>(
        deserializer: D,
    ) -> Result<BatteryFormatSettings, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into the struct directly
        let raw_settings = BatteryFormatSettings::deserialize(deserializer)?;

        // prepend each key from vars with a dollar sign
        // let dollar_prepended_vars = raw_settings.dollar_prepended_vars();
        let dollar_prepended_vars = raw_settings.vars.clone();

        // Apply formatting transformations
        let formatted = BatteryFormatSettings {
            vars: dollar_prepended_vars.clone(),
            full: safe_strfmt(&raw_settings.full, &dollar_prepended_vars),
            charging: safe_strfmt(&raw_settings.charging, &dollar_prepended_vars),
            not_charging: safe_strfmt(&raw_settings.not_charging, &dollar_prepended_vars),
            discharging: raw_settings.safe_strfmt_levels(&dollar_prepended_vars),
            default: safe_strfmt(&raw_settings.default, &dollar_prepended_vars),
        };

        Ok(formatted)
    }

}
