use std::{collections::HashMap, fs, path::PathBuf, time};

use anyhow::Context;
use serde::Deserialize;
use serde_yml::Value;
use smart_default::SmartDefault;

use crate::{deserialize_value, new_self_from_settings, utils};

use super::Component;

// Backlight //////////////////////////////////////////////////////////////////

#[derive(Debug, SmartDefault)]
pub struct Backlight {
    pub state: BacklightState,
    pub settings: BacklightSettings,
}

#[derive(Debug, SmartDefault)]
pub struct BacklightState {
    pub percent: Option<i32>,
    pub last_updated: Option<time::Instant>,
    pub cache: Option<String>,
}

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BacklightSettings {
    #[default(1000)]
    pub refresh_interval: u64,

    #[default(6)]
    pub signal: u32,

    #[default(PathBuf::from("/sys/class/backlight/acpi_video0"))]
    pub path: PathBuf,

    #[default(BacklightFormatSettings::default())]
    pub format: BacklightFormatSettings,
}

#[derive(Debug, SmartDefault, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BacklightFormatSettings {
    #[default("  {p} ")]
    pub default: String,

    #[default(None)]
    pub levels: Option<Vec<(i32, String)>>,
}

impl Component for Backlight {
    fn new_from_value(value: &Value) -> anyhow::Result<Self>
    where
        Self: std::marker::Sized,
    {
        let mut settings: BacklightSettings = deserialize_value!(value);

        // sort order of levels
        utils::sort_levels(&mut settings.format.levels);

        // create new Self from
        Ok(new_self_from_settings!(settings))
    }

    fn update_state(&mut self) -> anyhow::Result<()> {
        let path = &self.settings.path;
        let brightness: f32 = fs::read_to_string(path.join("brightness"))?
            .trim()
            .parse()?;
        let max_brightness: f32 = fs::read_to_string(path.join("max_brightness"))?
            .trim()
            .parse()?;
        let percent = ((brightness * 100.0) / max_brightness) as i32;

        self.state.percent = Some(percent);
        self.state.last_updated = Some(time::Instant::now());

        Ok(())
    }

    fn set_cache(&mut self, str: String) -> anyhow::Result<()> {
        self.state.cache = Some(str);
        Ok(())
    }

    fn get_strfmt_template(&self) -> anyhow::Result<Option<&str>> {
        let percent = &self.state.percent;
        let levels = &self.settings.format.levels;

        let template: Option<&str> = match (percent, levels) {
            // percent is None
            (None, _) => None,
            // levels is None, use default formatter
            (Some(_), None) => Some(&self.settings.format.default),
            // levels is Some
            // TODO: simplify with a function
            (Some(perc), Some(lvls)) => Some(
                lvls.iter()
                    .find(|(ceiling, _)| perc <= ceiling)
                    .map(|(_, format_str)| format_str)
                    .context("(N/A: could not find level)")?,
            ),
        };
        Ok(template)
    }

    fn apply_strfmt_template(&self, template: &str) -> anyhow::Result<Option<String>> {
        // TODO: simplify with a macro
        let vars: HashMap<String, String> = HashMap::from([(
            "p".to_owned(),
            match self.state.percent {
                Some(v) => v.to_string(),
                None => "N/A".to_string(),
            },
        )]);
        let res = strfmt::strfmt(template, &vars)?;

        Ok(Some(res))
    }

    fn get_last_updated(&self) -> anyhow::Result<&Option<std::time::Instant>> {
        Ok(&self.state.last_updated)
    }

    fn get_refresh_interval(&self) -> anyhow::Result<&u64> {
        Ok(&self.settings.refresh_interval)
    }

    fn get_cache(&self) -> anyhow::Result<&Option<String>> {
        Ok(&self.state.cache)
    }

    fn default_output(&self) -> anyhow::Result<&str> {
        Ok("N/A(default_output)")
    }
}
