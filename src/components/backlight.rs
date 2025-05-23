use std::{fs, path::PathBuf, time};

use anyhow::Context;
use serde::Deserialize;
use serde_yml::Value;
use smart_default::SmartDefault;

use crate::{apply_strfmt, impl_component_methods, new_from_value, utils::find_current_level};

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
    #[default(" br: {p} ")]
    pub default: String,

    #[default(None)]
    pub levels: Option<Vec<(i32, String)>>,
}

impl Component for Backlight {
    fn new_from_value(value: &Value) -> anyhow::Result<Self>
    where
        Self: std::marker::Sized,
    {
        new_from_value!(
            value => BacklightSettings,
            sort_levels: true
        )
    }

    fn update_state(&mut self) -> anyhow::Result<()> {
        let path = &self.settings.path;
        let brightness: f32 = fs::read_to_string(path.join("brightness"))
            .context("failed to read file brightness")?
            .trim()
            .parse()?;
        let max_brightness: f32 = fs::read_to_string(path.join("max_brightness"))
            .context("failed to read file max_brightness")?
            .trim()
            .parse()?;
        let percent = ((brightness * 100.0) / max_brightness).round() as i32;

        self.state.percent = Some(percent);
        self.state.last_updated = Some(time::Instant::now());

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
            (Some(perc), Some(lvls)) => Some(find_current_level(lvls, perc)?),
        };

        Ok(template)
    }

    fn apply_strfmt_template(&self, template: &str) -> anyhow::Result<Option<String>> {
        apply_strfmt!(
            template,
            "p" => match self.state.percent {
                Some(v) => v.to_string(),
                None => "N/A".to_string(),
            },
        )
    }

    impl_component_methods!(
        set_cache,
        get_last_updated,
        get_refresh_interval,
        get_signal_value,
        get_cache,
        default_output
    );
}
