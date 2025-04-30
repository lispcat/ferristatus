use std::{collections::HashMap, time};

use chrono::{DateTime, Local};
use serde::Deserialize;
use smart_default::SmartDefault;

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
}

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TimeSettings {
    #[default(500)]
    pub refresh_interval: u32,

    #[default(8)]
    pub signal: u32,

    #[default("%Y-%m-%d %H:%M:%S".to_string())]
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
    fn name(&self) -> String {
        "time".to_owned()
    }

    fn get_refresh_interval(&self) -> u32 {
        self.settings.refresh_interval
    }

    fn get_last_updated(&self) -> Option<std::time::Instant> {
        self.state.last_updated
    }

    fn update(&mut self) -> anyhow::Result<()> {
        self.state.now = Some(Local::now());
        Ok(())
    }

    fn get_format_str(&self) -> anyhow::Result<String> {
        Ok(self.settings.format.default.clone())
    }

    fn format(&self) -> anyhow::Result<String> {
        let format_string = &self.get_format_str()?;
        let time_fmt = &self.settings.time;
        let vars: HashMap<String, String> = HashMap::from([(
            "t".to_owned(),
            match self.state.now {
                Some(datetime) => datetime.format(time_fmt.as_str()).to_string(),
                None => "N/A".to_string(),
            },
        )]);
        Ok(strfmt::strfmt(format_string, &vars)?)
    }
}
