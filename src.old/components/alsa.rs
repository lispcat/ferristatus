use alsa_lib::mixer::{Mixer, Selem, SelemChannelId, SelemId};
use core::fmt;
use serde::Deserialize;
use smart_default::SmartDefault;
use std::{fmt::Display, time};

use crate::components::{Component, ComponentSettings, ComponentState};

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AlsaSettings {
    #[default(1000)]
    pub refresh_interval: u32,

    #[default(5)]
    pub signal: u32,

    pub volume_fmt: AlsaFmtSettings,

    pub muted_fmt: AlsaFmtSettings,
}

impl ComponentSettings for AlsaSettings {}

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AlsaFmtSettings {
    #[default(String::from(" "))]
    pub left_pad: String,

    #[default(String::from(" "))]
    pub right_pad: String,
}

#[derive(Debug, SmartDefault)]
pub struct AlsaState {
    pub volume_perc: Option<i32>,
    pub is_muted: Option<bool>,
    pub last_updated: Option<time::Instant>,
}
impl ComponentState for AlsaState {}

#[derive(Debug, SmartDefault)]
pub struct Alsa {
    pub state: AlsaState,
    pub settings: AlsaSettings,
}

impl Component for Alsa {
    fn name(&self) -> String {
        String::from("alsa")
    }
    fn update(&mut self) -> anyhow::Result<()> {
        // Open the default mixer
        let mixer: Mixer = Mixer::new("default", false).unwrap();

        // Get the Master control
        let selem_id: SelemId = SelemId::new("Master", 0);
        let selem: Selem<'_> = mixer.find_selem(&selem_id).unwrap();

        // Get volume range
        let (min, max) = selem.get_playback_volume_range();

        // Get current volume (first channel)
        let vol = selem
            .get_playback_volume(SelemChannelId::FrontLeft)
            .unwrap();
        let vol_perc_f = (vol as f64 - min as f64) / (max as f64 - min as f64) * 100.0;
        let vol_perc = vol_perc_f.round() as i32;

        // Get mute status
        let mute: bool = selem
            .get_playback_switch(SelemChannelId::FrontLeft)
            .unwrap()
            == 0;

        // update
        self.state.volume_perc = Some(vol_perc);
        self.state.is_muted = Some(mute);
        self.state.last_updated = Some(time::Instant::now());

        Ok(())
    }
    fn get_format_string(&self) -> String {
        todo!()
    }
    fn eval_strfmt(&self, format_str: &str) -> anyhow::Result<String> {
        todo!()
    }
}

impl Display for Alsa {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.state.volume_perc, self.state.is_muted) {
            (Some(vol), Some(is_muted)) => {
                if is_muted {
                    write!(
                        f,
                        "{}MUTE({}){}",
                        self.settings.muted_fmt.left_pad, vol, self.settings.muted_fmt.right_pad,
                    )
                } else {
                    write!(
                        f,
                        "{}{}{}",
                        self.settings.volume_fmt.left_pad, vol, self.settings.volume_fmt.right_pad
                    )
                }
            }
            _ => write!(f, "N/A"),
        }
    }
}

impl From<AlsaSettings> for Alsa {
    fn from(source: AlsaSettings) -> Self {
        Self {
            settings: source,
            ..Self::default()
        }
    }
}

// trait Test {
//     fn new_from_settings<T>(settings: T) -> Self
//     where
//         T: ComponentSettings,
//         Self: From<T>;
// }

// impl Test for Alsa {
//     fn new_from_settings<T>(settings: T) -> Self
//     where
//         T: ComponentSettings,
//         Self: From<T>
//     {
//         // You could use methods from ComponentSettings here if needed
//         Self::from(settings)
//     }
// }

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn alsa_current() {
    //     let mut alsa = Alsa::new();
    //     alsa.update().expect("failed to update alsa");
    //     println!(
    //         "> Alsa:\n\tcurrent: {:?},\n\tis_muted: {:?}",
    //         alsa.volume_perc, alsa.is_muted
    //     );
    // }
}
