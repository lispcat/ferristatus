use std::{collections::HashMap, fs, path::PathBuf, time};

use itertools::Itertools;
use serde::Deserialize;
use smart_default::SmartDefault;

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
}

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BacklightSettings {
    #[default(1000)]
    pub refresh_interval: u32,

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
    fn name(&self) -> String {
        "backlight".to_owned()
    }

    fn update(&mut self) -> anyhow::Result<()> {
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

    fn get_format_str(&self) -> anyhow::Result<String> {
        let percent = &self.state.percent;
        let levels = &self.settings.format.levels;

        match (percent, levels) {
            // percent is None
            (None, _) => Ok(
                "(N/A)".to_owned()
            ),
            // levels is None, use default formatter
            (Some(_), None) => Ok(
                self.settings.format.default.clone()
            ),
            // levels is Some
            (Some(perc), Some(lvls)) => Ok(
                lvls.iter()
                    .sorted_by(|a, b| a.0.cmp(&b.0))
                    .find(|(ceiling, _)| perc <= ceiling)
                    .map(|(_, format_str)| format_str.clone())
                    .unwrap_or("(N/A: could not find level)".to_owned())
            ),
        }
    }

    fn format(&self) -> anyhow::Result<String> {
        let format_string = &self.get_format_str()?;
        let vars: HashMap<String, String> = HashMap::from([
            ("p".to_owned(), match self.state.percent {
                Some(v) => v.to_string(),
                None => "N/A".to_string(),
            }),
        ]);
        Ok(strfmt::strfmt(format_string, &vars)?)
    }
}
