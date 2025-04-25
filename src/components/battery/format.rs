use std::collections::HashMap;

use crate::utils::safe_strfmt;
use serde::{Deserialize, Deserializer};
use smart_default::SmartDefault;

use crate::components::utils::de_vars_as_flat_hashmap;
use crate::components::ComponentFormat;

#[derive(Debug, SmartDefault, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryFormatSettings {
    #[default(HashMap::from([
        ("def".to_string(), "{percent}% {time_remaining}".to_string())
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

impl ComponentFormat for BatteryFormatSettings {
    fn de_strfmt<'de, D>(deserializer: D) -> Result<BatteryFormatSettings, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into the struct directly
        let fmt = BatteryFormatSettings::deserialize(deserializer)?;

        // prepend each key from vars with a dollar sign
        let vars = fmt.vars.clone();

        // Apply formatting transformations
        let formatted = BatteryFormatSettings {
            vars: vars.clone(),
            full: safe_strfmt(&fmt.full, &vars),
            charging: safe_strfmt(&fmt.charging, &vars),
            not_charging: safe_strfmt(&fmt.not_charging, &vars),
            discharging: fmt.safe_strfmt_levels(&vars),
            default: safe_strfmt(&fmt.default, &vars),
        };

        Ok(formatted)
    }

    fn get_levels(&self) -> Option<&Vec<(i32, String)>> {
        Some(&self.discharging)
    }
}
