use std::{collections::HashMap, path::PathBuf};

use acpi_client::{self, BatteryInfo, ChargingState};
use serde::{Deserialize, Deserializer};
use smart_default::SmartDefault;
use crate::utils::{de_vars_as_flat_hashmap, safe_strfmt};

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

    #[default(vec![])]
    pub subcomponents: Vec<String>,

    #[serde(deserialize_with = "BatteryFormatSettings::de_fields_with_vars_preprocessing")]
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

impl BatteryFormatSettings {
    pub fn de_fields_with_vars_preprocessing<'de, D>(
        deserializer: D,
    ) -> Result<BatteryFormatSettings, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into the struct directly
        let raw_settings = BatteryFormatSettings::deserialize(deserializer)?;

        let dollar_prepended_vars: HashMap<_, _> = raw_settings
            .vars
            .iter()
            .map(|(k, v)| (format!("${}", k), v.clone()))
            .collect();

        // Apply formatting transformations
        let formatted = BatteryFormatSettings {
            vars: dollar_prepended_vars.clone(),
            full: safe_strfmt(&raw_settings.full, &dollar_prepended_vars),
            charging: safe_strfmt(&raw_settings.charging, &dollar_prepended_vars),
            not_charging: safe_strfmt(&raw_settings.not_charging, &dollar_prepended_vars),
            discharging: {
                let mut levels: Vec<(i32, String)> = raw_settings
                    .discharging
                    .clone()
                    .into_iter()
                    .map(|(k, v)| (k, safe_strfmt(&v, &dollar_prepended_vars)))
                    .collect();
                levels.sort_by_key(|(k, _)| *k);
                levels
            },
            default: safe_strfmt(&raw_settings.default, &dollar_prepended_vars),
        };

        Ok(formatted)
    }

    /// gets the appropriate format string based off ChargingState
    pub fn get_format_string(&self, battery_info: &BatteryInfo) -> String {
        match battery_info.state {
            ChargingState::Full => self.full.clone(),
            ChargingState::Charging => self.charging.clone(),
            ChargingState::NotCharging => self.not_charging.clone(),
            ChargingState::Discharging => {
                let levels = self.discharging.clone();
                let percent = battery_info.percentage.round() as i32;

                levels
                    .iter()
                    .find(|(ceiling, _)| percent <= *ceiling)
                    .map(|(_, fmt_str)| fmt_str.clone())
                    .unwrap_or_else(|| "?".to_string())
            }
        }
    }
}

