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

    #[default(String::from(" "))]
    pub left_pad: String,

    #[default(String::from(" "))]
    pub right_pad: String,
}

impl ComponentSettings for TimeSettings {}

#[derive(Debug, SmartDefault)]
pub struct TimeState {
    pub now: Option<DateTime<Local>>,
}

#[derive(Debug, SmartDefault)]
pub struct Time {
    pub state: TimeState,
    pub settings: TimeSettings,
}

impl Time {
    // get time using Time.format
    pub fn get(&self) -> Result<String, Box<dyn Error>> {
        let now = self.state.now.ok_or("No timestamp available")?;
        let format = self.settings.format.as_str();

        Ok(now.format(format).to_string())
    }
}

impl Component for Time {
    fn name(&self) -> String {
        String::from("time")
    }
    // update
    fn update(&mut self) -> anyhow::Result<()> {
        self.state.now = Some(Local::now());
        Ok(())
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.state.now {
            Some(_) => {
                let formatted = self.get().unwrap();
                write!(
                    f,
                    "{}{}{}",
                    self.settings.left_pad, formatted, self.settings.right_pad
                )
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
