use std::time::Instant;

use acpi_client::{self, BatteryInfo};
use anyhow::Context;
use once_cell::sync::Lazy;
use regex::Regex;
use smart_default::SmartDefault;

use crate::components::ComponentState;

#[derive(Debug, SmartDefault)]
pub struct BatteryState {
    pub battery_info: Option<BatteryInfo>,
    pub last_updated: Option<Instant>,
}
impl ComponentState for BatteryState {}

impl BatteryState {
    /// Trims the trailing " [0-9]+s" from the end of a humantime::format_duration output.
    pub fn get_time_remaining(&self) -> anyhow::Result<String> {
        let duration = self.battery_info.as_ref().unwrap().time_remaining;
        let time = &humantime::format_duration(duration).to_string();
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s[0-9]+s$").unwrap());
        Ok(RE.replace(time, "").to_string())
    }

    /// get the current battery percent as a whole number
    pub fn get_percent_rounded(&self) -> anyhow::Result<String> {
        let battery_info = self
            .battery_info
            .as_ref()
            .context("could not get battery_info")?;
        let percent = battery_info.percentage.round() as i32;
        Ok(percent.to_string())
    }
}
