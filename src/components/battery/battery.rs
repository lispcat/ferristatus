use core::fmt;
use std::{collections::HashMap, fmt::Display, time::Instant};

use acpi_client::{self, BatteryInfo};
use anyhow::Context;
use smart_default::SmartDefault;
use strfmt::strfmt;

use crate::components::Component;

use super::{BatterySettings, BatteryState};

#[derive(Debug, SmartDefault)]
pub struct Battery {
    pub state: BatteryState,
    pub settings: BatterySettings,
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
