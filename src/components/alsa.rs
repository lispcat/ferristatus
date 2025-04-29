use std::{collections::HashMap, time};

use alsa_lib::{
    mixer::{Selem, SelemChannelId, SelemId},
    Mixer,
};
use anyhow::Context;
use itertools::Itertools;
use serde::Deserialize;
use smart_default::SmartDefault;

use super::{Component};

// Alsa ///////////////////////////////////////////////////////////////////////

#[derive(Debug, SmartDefault)]
pub struct Alsa {
    pub state: AlsaState,
    pub settings: AlsaSettings,
}

#[derive(Debug, SmartDefault)]
pub struct AlsaState {
    pub percent: Option<f64>,
    pub is_muted: Option<bool>,
    pub last_updated: Option<time::Instant>,
}

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AlsaSettings {
    #[default(1000)]
    pub refresh_interval: u32,

    #[default(5)]
    pub signal: u32,

    #[default(AlsaFormatSettings::default())]
    pub format: AlsaFormatSettings,
}

#[derive(Debug, SmartDefault, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AlsaFormatSettings {
    #[default(" A: {p}% ")]
    pub default: String,

    #[default(" MUTE({p}) ")]
    pub muted: String,

    #[default(None)]
    pub levels: Option<Vec<(i32, String)>>,
}

impl Component for Alsa {
    fn name(&self) -> String {
        "alsa".to_owned()
    }

    fn update(&mut self) -> anyhow::Result<()> {
        // Open the default mixer
        let mixer: Mixer = Mixer::new("default", false)
            .context("failed to open the default mixer")?;

        // Get the Master control
        let selem_id: SelemId = SelemId::new("Master", 0);
        let selem: Selem<'_> = mixer
            .find_selem(&selem_id)
            .context("failed to find selem for Master control")?;

        // Get volume range
        let (min, max) = selem.get_playback_volume_range();
        let (min, max) = (min as f64, max as f64);

        // Get current volume (first channel)
        let vol = selem
            .get_playback_volume(SelemChannelId::FrontLeft)
            .context("failed to get playback volume from selem")?
            as f64;
        let vol_percent_f = (vol - min) / (max - min) * 100.0;

        // Get mute status
        let mute: bool = selem
            .get_playback_switch(SelemChannelId::FrontLeft)
            .unwrap()
            == 0;

        // update
        self.state.percent = Some(vol_percent_f);
        self.state.is_muted = Some(mute);
        self.state.last_updated = Some(time::Instant::now());

        Ok(())
    }

    fn get_format_str(&self) -> anyhow::Result<String> {
        let percent = self.state.percent.map(|v| v as i32);
        let is_muted = self.state.is_muted;
        let levels = &self.settings.format.levels;

        // percent is None
        if percent.is_none() {
            return Ok("(N/A: unknown state: percent)".to_owned());
        }
        // is_muted is None
        if is_muted.is_none() {
            return Ok("(N/A: unknown state: is_muted)".to_owned());
        }
        // is_muted is Some(b)
        if is_muted.is_some_and(|b| b) {
            return Ok(self.settings.format.muted.clone());
        }

        match (percent, levels) {
            // percent is None
            (None, _) => Ok("(N/A)".to_owned()),
            // levels is None, use default formatter
            (Some(_), None) => Ok(self.settings.format.default.clone()),
            // levels is Some
            (Some(percent), Some(lvls)) => Ok(lvls
                .iter()
                .sorted_by(|a, b| a.0.cmp(&b.0))
                .find(|(ceiling, _)| &percent <= ceiling)
                .map(|(_, format_str)| format_str.clone())
                .unwrap_or("(N/A: could not find level)".to_owned())),
        }
    }

    fn format(&self) -> anyhow::Result<String> {
        let format_string = &self.get_format_str()?;
        let vars: HashMap<String, String> = HashMap::from([
            ("p".to_owned(), match self.state.percent {
                Some(v) => (v.round() as i64)
                    .to_string(),
                None => "N/A".to_string(),
            })
        ]);
        Ok(strfmt::strfmt(format_string, &vars)?)
    }
}
