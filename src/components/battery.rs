use core::fmt;
use std::{collections::HashMap, fmt::Display, time::Instant};
use acpi_client::{self, BatteryInfo, ChargingState};
use anyhow::Context;
use smart_default::SmartDefault;
use strfmt::strfmt;
use crate::components::Component;

pub mod format;
pub mod settings;
pub mod state;

pub use settings::*;
pub use state::*;

#[derive(Debug, SmartDefault)]
pub struct Battery {
    pub state: BatteryState,
    pub settings: BatterySettings,
}

// battery implementations

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

    /// gets the appropriate format string based off ChargingState
    fn get_format_string(&self) -> String {
        let battery_info = self.state.battery_info.as_ref().unwrap();
        let state = &battery_info.state;
        let fmt = &self.settings.format;
        match state {
            ChargingState::Full => fmt.full.clone(),
            ChargingState::Charging => fmt.charging.clone(),
            ChargingState::NotCharging => fmt.not_charging.clone(),
            ChargingState::Discharging => {
                let levels = fmt.discharging.clone();
                let percent = battery_info.percentage.round() as i32;

                levels
                    .iter()
                    .find(|(ceiling, _)| percent <= *ceiling)
                    .map(|(_, fmt_str)| fmt_str.clone())
                    .unwrap_or_else(|| "?".to_string())
            }
        }
    }

    fn eval_strfmt(&self, format_str: &str) -> anyhow::Result<String> {
        let mut vars = HashMap::new();

        vars.insert(
            "percent".to_owned(),
            self.state
                .get_percent_rounded()
                .unwrap_or_else(|e| format!("({e})")),
        );
        vars.insert(
            "time_remaining".to_string(),
            self.state
                .get_time_remaining()
                .unwrap_or_else(|e| format!("({e})")),
        );

        Ok(strfmt(format_str, &vars)?)
    }
}

impl Display for Battery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.state.battery_info {
            None => write!(f, "N/A"),
            Some(_) => {
                let format_string = self.get_format_string();

                let res = self.eval_strfmt(&format_string).map_err(|_| fmt::Error)?;

                write!(f, "{}", res)
            }
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
