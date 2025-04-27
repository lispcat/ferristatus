use std::collections::HashMap;

use crate::utils::safe_strfmt;
use serde::{de, Deserialize, Deserializer};
use smart_default::SmartDefault;

use crate::components::utils::de_vars_as_flat_hashmap;
use crate::components::ComponentFormatSettings;

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

impl ComponentFormatSettings for BatteryFormatSettings {
    fn de_strfmt<'de, D>(deserializer: D) -> Result<BatteryFormatSettings, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into the struct directly
        let format = BatteryFormatSettings::deserialize(deserializer)?;

        // prepend each key from vars with a dollar sign
        let vars = format.vars.clone();

        // Apply formatting transformations
        let formatted = BatteryFormatSettings {
            vars: vars.clone(),
            full: safe_strfmt(&format.full, &vars),
            charging: safe_strfmt(&format.charging, &vars),
            not_charging: safe_strfmt(&format.not_charging, &vars),
            discharging: format
                .safe_strfmt_levels(&vars)
                .map_err(|e| de::Error::custom(format!("{e}")))?,
            default: safe_strfmt(&format.default, &vars),
        };

        Ok(formatted)
    }

    fn get_levels(&self) -> Option<&Vec<(i32, String)>> {
        Some(&self.discharging)
    }
}

// TODO: reduce ComponentFormatSettings custom impls to just the formatted var.

trait ComponentFormatSettingsTest {
    fn get_vars(&self) -> &HashMap<String, String>;
    fn de_strfmt<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized + Deserialize<'de>,
    {
        // Deserialize into the struct directly
        let format = Self::deserialize(deserializer)?;

        let vars = format.get_vars().clone();

        // Apply formatting transformations
        let formatted = Self::de_strfmt_formatting(&format, vars).map_err(de::Error::custom)?;

        Ok(formatted)
    }

    fn de_strfmt_formatting<'de>(
        orig: &Self,
        vars: HashMap<String, String>,
    ) -> Result<Self, String>
    where
        Self: Sized;
}

impl ComponentFormatSettingsTest for BatteryFormatSettings {
    fn de_strfmt_formatting(
        orig: &Self,
        vars: HashMap<String, String>,
    ) -> Result<Self, String>
    where
        Self: Sized,
    {
        let new = BatteryFormatSettings {
            vars: vars.clone(),
            full: safe_strfmt(&orig.full, &vars),
            charging: safe_strfmt(&orig.charging, &vars),
            not_charging: safe_strfmt(&orig.not_charging, &vars),
            discharging: orig
                .safe_strfmt_levels(&vars)
                .map_err(|e| format!("{e}"))?,
            default: safe_strfmt(&orig.default, &vars),
        };
        Ok(new)
    }

    fn get_vars(&self) -> &HashMap<String, String> {
        &self.vars
    }
}
