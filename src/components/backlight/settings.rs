use serde::Deserialize;
use smart_default::SmartDefault;
use std::path::PathBuf;

use crate::components::ComponentSettings;

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BacklightSettings {
    #[default(1000)]
    pub refresh_interval: u32,

    #[default(6)]
    pub signal: u32,

    #[default(Box::from(PathBuf::from("/sys/class/backlight/acpi_video0")))]
    pub path: Box<PathBuf>,

    #[default(String::from(" "))]
    pub left_pad: String,

    #[default(String::from(" "))]
    pub right_pad: String,
}

impl ComponentSettings for BacklightSettings {}
