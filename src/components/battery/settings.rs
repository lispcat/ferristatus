use std::path::PathBuf;

use serde::Deserialize;
use smart_default::SmartDefault;

use crate::components::{ComponentFormatSettings, ComponentSettings};

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

pub mod format {

    use std::collections::HashMap;

    use crate::utils::safe_strfmt;
    use serde::Deserialize;
    use smart_default::SmartDefault;

    use crate::components::ComponentFormatSettings;
    use crate::components::utils::de_vars_as_flat_hashmap;

    #[derive(Debug, SmartDefault, Clone, Deserialize)]
    #[serde(default, deny_unknown_fields)]
    pub struct BatteryFormatSettings {
        #[default(HashMap::from([
            ("$def".to_string(), "{percent}% {time_remaining}".to_string())
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

    impl ComponentFormatSettings for BatteryFormatSettings {
        fn get_levels(&self) -> Option<&Vec<(i32, String)>> {
            Some(&self.discharging)
        }

        fn get_vars(&self) -> &HashMap<String, String> {
            &self.vars
        }

        fn de_strfmt_formatting(orig: &Self, vars: HashMap<String, String>) -> Result<Self, String>
        where Self: Sized {
            let new = BatteryFormatSettings {
                vars: vars.clone(),
                full: safe_strfmt(&orig.full, &vars),
                charging: safe_strfmt(&orig.charging, &vars),
                not_charging: safe_strfmt(&orig.not_charging, &vars),
                discharging: orig.safe_strfmt_levels(&vars).map_err(|e| format!("{e}"))?,
                default: safe_strfmt(&orig.default, &vars),
            };
            Ok(new)
        }
    }

}
