use core::fmt;
use serde::Deserialize;
use smart_default::SmartDefault;
use std::{fmt::Display, fs, path::PathBuf, time};

use super::{Component, ComponentSettings, ComponentState};

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BacklightSettings {
    #[default(1000)]
    pub refresh_interval: u32,

    #[default(6)]
    pub signal: u32,

    #[default(Box::from(PathBuf::from("/sys/class/backlight/acpi_video0")))]
    pub path: Box<PathBuf>,

    #[default(String::from(" "))]
    pub left_pad: String,

    #[default(String::from(" "))]
    pub right_pad: String,
}

impl ComponentSettings for BacklightSettings {}

#[derive(Debug, SmartDefault)]
pub struct BacklightState {
    pub perc: Option<i32>,
    pub last_updated: Option<time::Instant>,
}

impl ComponentState for BacklightState {}

#[derive(Debug, SmartDefault)]
pub struct Backlight {
    pub state: BacklightState,
    pub settings: BacklightSettings,
}

/// methods for fetching, parsing, and calculating
impl Backlight {
    // read values from fs, return values
    fn read_values_from_fs(&self) -> anyhow::Result<(f32, f32)> {
        let mut br_path = self.settings.path.clone();
        br_path.push("brightness");
        let br_read: String = fs::read_to_string(*br_path)?;
        let br: f32 = br_read.trim().parse()?;

        let mut max_br_path = self.settings.path.clone();
        max_br_path.push("max_brightness");
        let max_br_read: String = fs::read_to_string(*max_br_path)?;
        let max_br: f32 = max_br_read.trim().parse()?;

        Ok((br, max_br))
    }
}

impl Component for Backlight {
    fn name(&self) -> String {
        String::from("backlight")
    }
    // update
    fn update(&mut self) -> anyhow::Result<()> {
        let (brightness, max_brightness) = self.read_values_from_fs()?;
        self.state.perc = Some(calc_percent_from_values(brightness, max_brightness));
        self.state.last_updated = Some(time::Instant::now());
        Ok(())
    }
}

impl Display for Backlight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.state.perc {
            Some(perc) => write!(
                f,
                "{}{}{}",
                self.settings.left_pad, perc, self.settings.right_pad
            ),
            None => write!(f, "N/A"),
        }
    }
}

// pure function, simply calculate percent from values
fn calc_percent_from_values(brightness: f32, max_brightness: f32) -> i32 {
    ((brightness * 100.0) / max_brightness) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// test whether a given br and max_br value results in the exected perc
    #[test]
    fn percent_calc() {
        assert_eq!(calc_percent_from_values(50.0, 100.0), 50);
        assert_eq!(calc_percent_from_values(75.0, 100.0), 75);
        assert_eq!(calc_percent_from_values(100.0, 200.0), 50);
        assert_eq!(calc_percent_from_values(0.0, 100.0), 0);
        assert_eq!(calc_percent_from_values(100.0, 100.0), 100);
    }

    //     /// simply print the current backlight percent. use `-- --nocapture`.
    //     #[test]
    //     fn current_percent() {
    //         let mut bl: Backlight = Default::default();
    //         bl.update().expect("could not update Backlight");
    //         println!(
    //             "> Current Backlight:
    // \t{:?}
    // \t{:?}
    // \t{:?}",
    //             bl.settings.path, bl.perc, bl.last_updated
    //         );
    //     }
}
