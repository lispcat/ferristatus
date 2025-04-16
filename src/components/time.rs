use core::fmt;
use std::{error::Error, fmt::Display};

use anyhow::Result;
use chrono::{DateTime, Local};
use serde::Deserialize;
use smart_default::SmartDefault;

use super::{Component, ComponentSettings};

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TimeSettings {
    #[default(500)]
    pub refresh_interval: u32,

    #[default(8)]
    pub signal: u32,

    #[default("%Y-%m-%d %H:%M:%S".to_string())]
    pub format: String,
}

impl ComponentSettings for TimeSettings {}

#[derive(Debug, SmartDefault)]
pub struct Time {
    pub now: Option<DateTime<Local>>,
    pub settings: TimeSettings,
}

impl Time {
    // get time using Time.format
    pub fn get(&self) -> Result<String, Box<dyn Error>> {
        let now = self.now.ok_or("No timestamp available")?;
        let format = self.settings.format.as_str();

        Ok(now.format(format).to_string())
    }
}

impl Component for Time {
    // update
    fn update(&mut self) -> anyhow::Result<()> {
        self.now = Some(Local::now());
        Ok(())
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.now {
            Some(_) => {
                let time = self.get().is_ok();
                write!(f, "\\{}\\", time)
            }
            None => write!(f, "N/A"),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test() {
//         let mut time = Time::new();
//         time.update();
//         println!("> Time:\n\t{:?}\n\t{:?}", time.settings.format, time.get());
//     }
// }
