use core::fmt;
use std::{
    collections::HashMap, fmt::Display, hash::Hash, path::PathBuf, str::FromStr, time::Instant,
};

use acpi_client::{self, BatteryInfo, ChargingState};
use anyhow::Context;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use smart_default::SmartDefault;
use strfmt::{strfmt, DisplayStr};

use super::{Component, ComponentSettings, ComponentState};

#[derive(Debug, SmartDefault)]
pub struct Battery {
    pub state: BatteryState,
    pub settings: BatterySettings,
}

#[derive(Debug, SmartDefault)]
pub struct BatteryState {
    pub battery_info: Option<BatteryInfo>,
    pub last_updated: Option<Instant>,
}
impl ComponentState for BatteryState {}

impl BatteryState {
    /// Trims the trailing " [0-9]+s" from the end of a humantime::format_duration output.
    fn get_time_remaining(&self) -> anyhow::Result<String> {
        let duration = self.battery_info.as_ref().unwrap().time_remaining;
        let time = &humantime::format_duration(duration).to_string();
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s[0-9]+s$").unwrap());
        Ok(RE.replace(time, "").to_string())
    }

    /// get the current battery percent as a whole number
    fn get_percent_rounded(&self) -> anyhow::Result<String> {
        let battery_info = self
            .battery_info
            .as_ref()
            .context("could not get battery_info")?;
        let percent = battery_info.percentage.round() as i32;
        Ok(percent.to_string())
    }
}

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

    #[serde(deserialize_with = "BatteryFormatSettings::deserialize_with_vars_preprocessing")]
    pub format: BatteryFormatSettings,
}
impl ComponentSettings for BatterySettings {}

#[derive(Debug, SmartDefault, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryFormatSettings {
    #[default(HashMap::from([
        ("def".to_string(), "{percent}% {time_remaining}".to_string())
    ]))]
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
}

impl BatteryFormatSettings {
    fn deserialize_with_vars_preprocessing<'de, D>(
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
        };

        Ok(formatted)
    }

    /// gets the appropriate format string based off ChargingState
    fn get_format_string(&self, battery_info: &BatteryInfo) -> String {
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

// battery implementations

impl Battery {}

impl Component for Battery {
    fn name(&self) -> String {
        String::from("battery")
    }
    /// Updates the Battery struct
    fn update(&mut self) -> anyhow::Result<()> {
        let dir = self.settings.path.clone().into_boxed_path();
        let ps_info = BatteryInfo::new(&dir)
            .with_context(|| "failed to to create new BatteryInfo instance")?;
        self.state.battery_info = Some(ps_info);
        self.state.last_updated = Some(Instant::now());

        Ok(())
    }
}

impl Display for Battery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.state.battery_info {
            None => write!(f, "N/A"),
            Some(battery_info) => {
                let format_string = self.settings.format.get_format_string(battery_info);

                let vars = HashMap::from([
                    (
                        "percent".to_string(),
                        self.state
                            .get_percent_rounded()
                            .unwrap_or_else(|e| format!("N/A ({})", e)),
                    ),
                    (
                        "time_remaining".to_string(),
                        self.state
                            .get_time_remaining()
                            .unwrap_or_else(|e| format!("N/A ({})", e)),
                    ),
                ]);

                let res = strfmt(&format_string, &vars).unwrap();

                write!(f, "{}", res)
            }
        }
    }
}

fn safe_strfmt<K, T: DisplayStr>(template: &str, vars: &HashMap<K, T>) -> String
where
    K: Hash + Eq + FromStr + Display,
{
    match strfmt(template, vars) {
        Ok(formatted) => formatted,
        Err(_) => {
            // Return the original template
            template.to_string()
        }
    }
}

//// testing

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn updating() {
//         let mut bat = Battery::new();
//         bat.update().expect("failed to update battery struct");
//         println!("> Battery: {:#?}", bat);
//     }
// }
