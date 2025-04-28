use crate::components::Component;
use core::fmt;
use smart_default::SmartDefault;
use state::BacklightState;
use std::{collections::HashMap, fmt::Display, fs, time};
use strfmt::strfmt;

pub mod settings;
pub mod state;

pub use settings::*;
// pub use state::*;

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
    fn get_format_string(&self) -> String {
        let format = &self.settings.format;
        match self.state.perc {
            None => "N/A".to_owned(),
            Some(percent) => {
                let levels = format.levels.clone();

                let res = levels
                    .iter()
                    .find(|(ceiling, _)| percent <= *ceiling)
                    .map(|(_, format_str)| format_str.clone())
                    .unwrap_or_else(|| "?".to_string());
                res
            },
        }
    }
    fn eval_strfmt(&self, format_str: &str) -> anyhow::Result<String> {
        let mut vars: HashMap<String, String> = HashMap::new();

        vars.insert(
            "percent".to_owned(),
            match self.state.perc {
                Some(p) => p.to_string(),
                None => "N/A".to_string(),
            },
        );

        let res = strfmt(format_str, &vars)
            .unwrap_or_else(|e| format!("({})", e));
        Ok(res)
    }
}

impl Display for Backlight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.state.perc {
            None => write!(f, "{}", "N/A".to_owned()),
            Some(_) => {
                let format_string = self.get_format_string();

                let res = self.eval_strfmt(&format_string).map_err(|_| fmt::Error)?;

                write!(f, "{}", res)
            }
        }
    }
}

impl From<BacklightSettings> for Backlight {
    fn from(source: BacklightSettings) -> Self {
        Self {
            settings: source,
            ..Self::default()
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
