use format::BacklightFormatSettings;
use serde::Deserialize;
use smart_default::SmartDefault;
use std::path::PathBuf;

use crate::components::ComponentFormatSettings;
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

    #[serde(deserialize_with = "BacklightFormatSettings::de_strfmt")]
    pub format: BacklightFormatSettings,
}

impl ComponentSettings for BacklightSettings {}

pub mod format {

    use std::collections::HashMap;

    use serde::Deserialize;
    use smart_default::SmartDefault;

    use crate::components::utils::de_vars_as_flat_hashmap;
    use crate::components::ComponentFormatSettings;
    use crate::utils::safe_strfmt;

    #[derive(Debug, SmartDefault, Clone, Deserialize)]
    #[serde(default, deny_unknown_fields)]
    pub struct BacklightFormatSettings {
        #[default(HashMap::from([
            ("$def".to_string(), "{percent}%".to_string())
        ]))]
        #[serde(deserialize_with = "de_vars_as_flat_hashmap")]
        pub vars: HashMap<String, String>,

        #[default(vec![
            (100, "  {percent} ".to_string()),
        ])]
        pub levels: Vec<(i32, String)>,

        #[default("  {$def} ")]
        pub default: String,
    }

    impl ComponentFormatSettings for BacklightFormatSettings {
        fn get_vars(&self) -> &std::collections::HashMap<String, String> {
            &self.vars
        }

        fn get_levels(&self) -> Option<&Vec<(i32, String)>> {
            Some(&self.levels)
        }

        fn de_strfmt_formatting<'de>(
            orig: &Self,
            vars: std::collections::HashMap<String, String>,
        ) -> Result<Self, String>
        where
            Self: Sized,
        {
            let new = BacklightFormatSettings {
                vars: vars.clone(),
                levels: orig.safe_strfmt_levels(&vars).map_err(|e| format!("{e}"))?,
                default: safe_strfmt(&orig.default, &vars),
            };

            Ok(new)
        }
    }
}
