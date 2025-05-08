use std::time;

use alsa_lib::{
    mixer::{Selem, SelemChannelId, SelemId},
    Mixer,
};
use anyhow::Context;
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::utils::{apply_strfmt, deserialize_value, find_current_level, impl_component_methods, new_from_value};

use super::Component;

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
    pub cache: Option<String>,
}

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AlsaSettings {
    #[default(1000)]
    pub refresh_interval: u64,

    #[default(5)]
    pub signal: u32,

    #[default(AlsaFormatSettings::default())]
    pub format: AlsaFormatSettings,
}

#[derive(Debug, SmartDefault, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AlsaFormatSettings {
    #[default(" V: {p}% ")]
    pub default: String,

    #[default(" MUTE({p}) ")]
    pub muted: String,

    #[default(None)]
    pub levels: Option<Vec<(i32, String)>>,
}

impl Component for Alsa {
    fn new_from_value(value: &serde_yml::Value) -> anyhow::Result<Self>
    where
        Self: std::marker::Sized,
    {
        new_from_value!(
            value => AlsaSettings,
            sort_levels: true
        )
    }

    fn update_state(&mut self) -> anyhow::Result<()> {
        // Open the default mixer
        let mixer = Mixer::new("default", false).context("failed to open the default mixer")?;

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
            .context("failed to get playback volume from selem")? as f64;
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

    fn get_strfmt_template(&self) -> anyhow::Result<Option<&str>> {
        let percent = &self.state.percent.map(|v| v as i32);
        let is_muted = &self.state.is_muted;
        let levels = &self.settings.format.levels;

        // percent is None
        if percent.is_none() {
            return Ok(Some("(N/A: unknown state: percent)"));
        }
        // is_muted is None
        if is_muted.is_none() {
            return Ok(Some("(N/A: unknown state: is_muted)"));
        }
        // is_muted is Some(b)
        if is_muted.is_some_and(|b| b) {
            return Ok(Some(self.settings.format.muted.as_str()));
        }

        let template: Option<&str> = match (percent, levels) {
            // percent is None
            (None, _) => Some("(N/A)"),
            // levels is None, use default formatter
            (Some(_), None) => Some(self.settings.format.default.as_str()),
            // levels is Some
            (Some(percent), Some(lvls)) => Some(
                find_current_level(lvls, percent)?
            ),
        };
        Ok(template)
    }

    fn apply_strfmt_template(&self, template: &str) -> anyhow::Result<Option<String>> {
        apply_strfmt!(
            template,
            "p" => match self.state.percent {
                Some(v) => (v.round() as i64).to_string(),
                None => "N/A".to_string(),
            },
        )
    }

    impl_component_methods!(
        set_cache,
        get_last_updated,
        get_refresh_interval,
        get_cache,
        default_output
    );
}
