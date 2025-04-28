use std::{collections::HashMap, path::PathBuf, time};

use acpi_client::{BatteryInfo, ChargingState};
use anyhow::Context;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use smart_default::SmartDefault;

use super::Component;

#[derive(Debug, SmartDefault)]
pub struct Battery {
    pub state: BatteryState,
    pub settings: BatterySettings,
}

impl Component for Battery {
    fn name(&self) -> String {
        "battery".to_owned()
    }

    fn update(&mut self) -> anyhow::Result<()> {
        let path = &self.settings.path;
        let battery_info = BatteryInfo::new(path)
            .context("failed to create new BatteryInfo instance")?;

        self.state.battery_info = Some(battery_info);
        self.state.last_updated = Some(time::Instant::now());
        Ok(())
    }

    fn get_format_str(&self) -> anyhow::Result<String> {
        let format_settings = &self.settings.format;
        let battery_info = match &self.state.battery_info {
            Some(b) => b,
            None => anyhow::bail!("failed to get battery_info")
        };

        match battery_info.state {
            ChargingState::Full => Ok(format_settings.full.clone()),
            ChargingState::Charging => Ok(format_settings.charging.clone()),
            ChargingState::NotCharging => Ok(format_settings.not_charging.clone()),
            ChargingState::Discharging => {
                let levels = &self.settings.format.discharging;
                match levels {
                    None => Ok(format_settings.default.clone()),
                    Some(lvls) => {
                        let percent = battery_info.percentage.round() as i32;
                        let res = lvls
                            .iter()
                            .sorted_by(|a, b| a.0.cmp(&b.0))
                            .find(|(ceiling, _)| percent <= *ceiling)
                            .map(|(_, format_str)| format_str.clone())
                            .unwrap_or_else(|| "(N/A: could not find level)".to_owned());
                        Ok(res)
                    }
                }
            }
        }
    }

    fn format(&self) -> anyhow::Result<String> {
        let format_string = self.get_format_str()?;
        let vars: HashMap<String, String> = HashMap::from([
            ("p".to_owned(), match &self.state.battery_info {
                Some(b) => (b.percentage.round() as i32).to_string(),
                None => "N/A".to_string(),
            }),
            ("t".to_owned(), match &self.state.battery_info {
                None => "N/A".to_string(),
                Some(b) => {
                    let duration = b.time_remaining;
                    let time = &humantime::format_duration(duration).to_string();
                    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s[0-9]+s$").unwrap());
                    RE.replace(time, "").to_string()
                }
            }),
        ]);
        Ok(strfmt::strfmt(&format_string, &vars)?)
    }
}

#[derive(Debug, SmartDefault)]
pub struct BatteryState {
    pub battery_info: Option<BatteryInfo>,
    pub last_updated: Option<time::Instant>,
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

    #[default(BatteryFormatSettings::default())]
    pub format: BatteryFormatSettings,
}

#[derive(Debug, SmartDefault, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryFormatSettings {
    #[default(" {p}% {t} ")]
    pub default: String,

    #[default(" Full({p}) ".to_string())]
    pub full: String,

    #[default(" ? {p}% ".to_string())]
    pub not_charging: String,

    #[default("  {p}% {t} ".to_string())]
    pub charging: String,

    #[default(Some(vec![
        (100, " Full({p}) ".to_string()),
        (99,  "  {p}% {t} ".to_string()),
        (70,  "  {p}% {t} ".to_string()),
        (50,  "  {p}% {t} ".to_string()),
        (30,  "  {p}% {t} ".to_string()),
        (10,  "  {p}% {t} ".to_string()),
    ]))]
    pub discharging: Option<Vec<(i32, String)>>,
}
