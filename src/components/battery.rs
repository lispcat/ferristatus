use core::fmt;
use std::{fmt::Display, path::PathBuf, time::Instant};

use acpi_client::{self, BatteryInfo};
use anyhow::Context;
use serde::Deserialize;
use smart_default::SmartDefault;

use super::{Component, ComponentSettings};

/// Settings for the Battery component.
/// Typically configured through the config file.
#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatterySettings {
    #[default(1000)]
    pub refresh_interval: u32,

    #[default(7)]
    pub signal: u32,

    #[default(PathBuf::from("/sys/class/power_supply/BAT0"))]
    pub path: PathBuf,

    #[default(Vec::new())]
    pub subcomponents: Vec<String>,

    pub percent: BatterySubcomponentSettings,

    pub time_left: BatterySubcomponentSettings,
}

impl ComponentSettings for BatterySettings {}

/// Subcomponents for BatterySettings
#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatterySubcomponentSettings {
    #[default(String::from(" "))]
    pub left_pad: String,

    #[default(String::from(" "))]
    pub right_pad: String,
}

/// Holds current battery state and BatterySettings
#[derive(Debug, SmartDefault)]
pub struct Battery {
    pub battery_info: Option<BatteryInfo>,
    pub last_updated: Option<Instant>,
    pub settings: BatterySettings,
}

// Make Battery a Component

impl Component for Battery {
    // update
    fn update(&mut self) -> anyhow::Result<()> {
        let dir = self.settings.path.clone().into_boxed_path();
        let ps_info = BatteryInfo::new(&dir)
            .with_context(|| "failed to to create new BatteryInfo instance")?;
        self.battery_info = Some(ps_info);
        self.last_updated = Some(Instant::now());

        Ok(())
    }
}

impl Display for Battery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.battery_info {
            Some(battery) => write!(
                f,
                "{}{}{}",
                self.settings.percent.left_pad, battery.percentage, self.settings.percent.right_pad
            ),
            None => write!(f, "N/A"),
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
