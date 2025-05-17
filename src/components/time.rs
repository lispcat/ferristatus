use std::time;

use chrono::{DateTime, Local};
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::{apply_strfmt, impl_component_methods, new_from_value};

use super::Component;

// Time ///////////////////////////////////////////////////////////////////////

#[derive(Debug, SmartDefault)]
pub struct Time {
    pub state: TimeState,
    pub settings: TimeSettings,
}

#[derive(Debug, SmartDefault)]
pub struct TimeState {
    pub now: Option<DateTime<Local>>,
    pub last_updated: Option<time::Instant>,
    pub cache: Option<String>,
}

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TimeSettings {
    #[default(1000)]
    pub refresh_interval: u64,

    #[default(9)]
    pub signal: u32,

    #[default("%a %d %b %I:%M %P".to_string())]
    pub time: String,

    #[default(TimeFormatSettings::default())]
    pub format: TimeFormatSettings,
}

#[derive(Debug, SmartDefault, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TimeFormatSettings {
    #[default(" {t} ")]
    pub default: String,
}

impl Component for Time {
    fn new_from_value(value: &serde_yml::Value) -> anyhow::Result<Self>
    where
        Self: std::marker::Sized,
    {
        new_from_value!(
            value => TimeSettings
        )
    }

    fn update_state(&mut self) -> anyhow::Result<()> {
        self.state.now = Some(Local::now());
        Ok(())
    }

    fn get_strfmt_template(&self) -> anyhow::Result<Option<&str>> {
        Ok(Some(&self.settings.format.default))
    }

    fn apply_strfmt_template(&self, template: &str) -> anyhow::Result<Option<String>> {
        apply_strfmt!(
            template,
            "t" => match self.state.now {
                Some(datetime) => datetime.format(&self.settings.time).to_string(),
                None => "N/A".to_string(),
            }
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
