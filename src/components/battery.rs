use std::{error::Error, fmt, path::PathBuf, time::Instant};

use acpi_client::{self, BatteryInfo, ChargingState};
use anyhow::Result;
use serde::Deserialize;
use smart_default::SmartDefault;

// implement the Debug trait using newtype wrappers for
// some custom data types in acpi_client

// pub struct DebugChargingState(ChargingState);

// impl fmt::Debug for DebugChargingState {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self.0 {
//             ChargingState::Charging => write!(f, "Charging"),
//             ChargingState::Discharging => write!(f, "Discharging"),
//             ChargingState::Full => write!(f, "Full"),
//         }
//     }
// }

// pub struct DebugBatteryInfo(BatteryInfo);

// impl fmt::Debug for DebugBatteryInfo {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.debug_struct("BatteryInfo")
//             .field("name", &self.0.name)
//             .field("remaining_capacity", &self.0.remaining_capacity)
//             .field("present_rate", &self.0.present_rate)
//             .field("voltage", &self.0.voltage)
//             .field("design_capacity", &self.0.design_capacity)
//             .field("last_capacity", &self.0.last_capacity)
//             .field("time_remaining", &self.0.time_remaining) // Duration implements Debug
//             .field("percentage", &self.0.percentage)
//             .field("state", &DebugChargingState(self.0.state)) // Wrap ChargingState for Debug
//             .finish()
//     }
// }

// Battery struct

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

    #[default(None)]
    pub percent: Option<BatterySubcomponentSettings>,

    #[default(None)]
    pub time_left: Option<BatterySubcomponentSettings>,
}

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatterySubcomponentSettings {
    #[default(String::from(" "))]
    pub left_pad: String,

    #[default(String::from(" "))]
    pub right_pad: String,
}

#[derive(Debug, SmartDefault)]
pub struct Battery {
    pub battery_info: Option<BatteryInfo>,
    pub last_updated: Option<Instant>,
    pub settings: BatterySettings,
}

// battery methods

impl Battery {
    // initialization
    pub fn new() -> Self {
        Self::default()
    }

    // update
    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let dir = self.settings.path.clone().into_boxed_path();
        let ps_info = BatteryInfo::new(&dir).expect("failed to to create new BatteryInfo instance");
        self.battery_info = Some(ps_info);
        self.last_updated = Some(Instant::now());

        Ok(())
    }
}

// testing

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updating() {
        let mut bat = Battery::new();
        bat.update().expect("failed to update battery struct");
        println!("> Battery: {:#?}", bat);
    }
}
