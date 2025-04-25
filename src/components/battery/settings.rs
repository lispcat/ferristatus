use std::path::PathBuf;

use serde::Deserialize;
use smart_default::SmartDefault;

use crate::components::{ComponentFormat, ComponentSettings};

use super::format::BatteryFormatSettings;

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
