use std::error::Error;

use anyhow::Result;
use chrono::{DateTime, Local};
use serde::Deserialize;
use smart_default::SmartDefault;

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TimeSettings {
    #[default(500)]
    pub refresh_interval: u32,

    #[default(8)]
    pub signal: u32,

    #[default(Some("%Y-%m-%d %H:%M:%S".to_string()))]
    pub format: Option<String>,
}

#[derive(Debug, SmartDefault)]
pub struct Time {
    pub now: Option<DateTime<Local>>,
    pub settings: TimeSettings,
}

impl Time {
    // init
    pub fn new() -> Self {
        Default::default()
    }

    // update
    pub fn update(&mut self) {
        self.now = Some(Local::now());
    }

    // get time using Time.format
    pub fn get(&self) -> Result<String, Box<dyn Error>> {
        let now = self.now.ok_or("No timestamp available")?;
        let format = match &self.settings.format {
            Some(s) => s.as_str(),
            None => return Err("No time format string specified".into()),
        };

        Ok(now.format(format).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut time = Time::new();
        time.update();
        println!("> Time:\n\t{:?}\n\t{:?}", time.settings.format, time.get());
    }
}
