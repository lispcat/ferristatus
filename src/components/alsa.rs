use std::time;

use alsa_lib::{
    mixer::{Selem, SelemChannelId, SelemId},
    Mixer,
};
use anyhow::Context;
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::{apply_strfmt, impl_component_methods, utils::find_current_level};

use super::Component;

// Alsa ///////////////////////////////////////////////////////////////////////

#[derive(Debug, SmartDefault)]
pub struct Alsa {
    pub state: AlsaState,
    pub settings: AlsaSettings,
}

#[derive(Debug, SmartDefault)]
pub struct AlsaState {
    pub percent: Option<i64>,
    pub is_muted: Option<bool>,
    pub last_updated: Option<time::Instant>,
    pub cache: Option<String>,

    pub mixer: Option<Mixer>,
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
        let mut new = Self {
            settings: crate::deserialize_value!(value),
            ..Self::default()
        };

        new.state.mixer =
            Some(Mixer::new("default", false).context("failed to open the default mixer")?);

        Ok(new)
    }

    fn update_state(&mut self) -> anyhow::Result<()> {
        // Get the Master control
        let mixer = self.state.mixer.as_ref().context("mixer is none")?;

        // refresh the mixer
        mixer.handle_events().ok();

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
        self.state.percent = Some(vol_percent_f.round() as i64);
        self.state.is_muted = Some(mute);
        self.state.last_updated = Some(time::Instant::now());

        Ok(())
    }

    fn get_strfmt_template(&self) -> anyhow::Result<Option<&str>> {
        let percent = &self.state.percent.map(|v| v as i32);
        let is_muted = &self.state.is_muted;
        let levels = &self.settings.format.levels;

        let template: Option<&str> = match (is_muted, percent, levels) {
            // is_muted is None
            (None, _, _) => Some("N/A: (is_muted is None)"),
            // is_muted is Some(true)
            (Some(true), _, _) => Some(self.settings.format.muted.as_str()),

            // percent is None
            (_, None, _) => Some("(N/A)"),

            // percent is Some, no levels
            (_, Some(_), None) => Some(self.settings.format.default.as_str()),

            // percent is Some, yes levels
            (_, Some(percent), Some(lvls)) => Some(find_current_level(lvls, percent)?),
        };

        Ok(template)
    }

    fn apply_strfmt_template(&self, template: &str) -> anyhow::Result<Option<String>> {
        apply_strfmt!(
            template,
            "p" => match self.state.percent {
                Some(v) => v.to_string(),
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
