use alsa::mixer::{Mixer, Selem, SelemId};
use anyhow::Result;
use serde::Deserialize;
use smart_default::SmartDefault;
use std::{error::Error, time};

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AlsaSettings {
    #[default(1000)]
    pub refresh_interval: u32,

    #[default(5)]
    pub signal: u32,

    #[default(String::from(" "))]
    pub left_pad: String,

    #[default(String::from(" "))]
    pub right_pad: String,
}

#[derive(Debug, SmartDefault)]
pub struct Alsa {
    pub volume_perc: Option<i32>,
    pub is_muted: Option<bool>,
    pub last_updated: Option<time::Instant>,
    pub settings: AlsaSettings,
}

impl Alsa {
    // initialization
    pub fn new() -> Self {
        Default::default()
    }

    fn update(&mut self) -> Result<(), Box<dyn Error>> {
        // Open the default mixer
        let mixer: Mixer = Mixer::new("default", false).unwrap();

        // Get the Master control
        let selem_id: SelemId = SelemId::new("Master", 0);
        let selem: Selem<'_> = mixer.find_selem(&selem_id).unwrap();

        // Get volume range
        let (min, max) = selem.get_playback_volume_range();

        // Get current volume (first channel)
        let vol = selem
            .get_playback_volume(alsa::mixer::SelemChannelId::FrontLeft)
            .unwrap();
        let vol_perc_f = (vol as f64 - min as f64) / (max as f64 - min as f64) * 100.0;
        let vol_perc = vol_perc_f.round() as i32;

        // Get mute status
        let mute: bool = selem
            .get_playback_switch(alsa::mixer::SelemChannelId::FrontLeft)
            .unwrap()
            == 0;

        // update
        self.volume_perc = Some(vol_perc);
        self.is_muted = Some(mute);
        self.last_updated = Some(time::Instant::now());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alsa_current() {
        let mut alsa = Alsa::new();
        alsa.update().expect("failed to update alsa");
        println!(
            "> Alsa:\n\tcurrent: {:?},\n\tis_muted: {:?}",
            alsa.volume_perc, alsa.is_muted
        );
    }
}
