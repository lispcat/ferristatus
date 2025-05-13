use std::{path::PathBuf, time};

use acpi_client::{BatteryInfo, ChargingState};
use anyhow::Context;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use smart_default::SmartDefault;

use crate::{apply_strfmt, impl_component_methods, utils::find_current_level};

use super::Component;

// Battery ////////////////////////////////////////////////////////////////////

#[derive(Debug, SmartDefault)]
pub struct Battery {
    pub state: BatteryState,
    pub settings: BatterySettings,
}

#[derive(Debug, SmartDefault)]
pub struct BatteryState {
    pub percent: Option<i32>,
    pub time: Option<time::Duration>,
    pub charging_state: Option<ChargingState>,
    pub last_updated: Option<time::Instant>,
    pub cache: Option<String>,
}

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatterySettings {
    #[default(1000)]
    pub refresh_interval: u64,

    #[default(7)]
    pub signal: u32,

    #[default(PathBuf::from("/sys/class/power_supply/BAT0"))]
    pub path: PathBuf,

    #[default(BatteryFormatSettings::default())]
    pub format: BatteryFormatSettings,
}

#[derive(Debug, SmartDefault, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryFormatSettings {
    #[default(" B: {p}% {t} ")]
    pub default: String,

    #[default(" Full({p}) ")]
    pub full: String,

    #[default(" ? {p}% ")]
    pub not_charging: String,

    #[default("  {p}% {t} ")]
    pub charging: String,

    #[default(None)]
    pub discharging: Option<Vec<(i32, String)>>,
}

impl Component for Battery {
    fn new_from_value(value: &serde_yml::Value) -> anyhow::Result<Self>
    where
        Self: std::marker::Sized
    {
        {
            let mut settings:BatterySettings = crate::deserialize_value!(value);
            if true {
                crate::utils::sort_levels(&mut settings.format.discharging);
            }
            Ok(Self {
                settings,
                ..Self::default()
            })
        }
    }

    fn update_state(&mut self) -> anyhow::Result<()> {
        let path = &self.settings.path;
        let battery_info = BatteryInfo::new(path)
            .context("failed to create new BatteryInfo instance")?;

        self.state.percent = Some(battery_info.percentage.round() as i32);
        self.state.time = Some(battery_info.time_remaining);
        self.state.charging_state = Some(battery_info.state);

        self.state.last_updated = Some(time::Instant::now());

        Ok(())
    }

    fn get_strfmt_template(&self) -> anyhow::Result<Option<&str>> {
        let format_settings = &self.settings.format;
        let charging_state = self.state.charging_state
            .context("no charging state")?;

        let template: Option<&str> = match charging_state {
            ChargingState::Full => Some(
                &format_settings.full
            ),
            ChargingState::Charging => Some(
                &format_settings.charging
            ),
            ChargingState::NotCharging => Some(
                &format_settings.not_charging
            ),
            ChargingState::Discharging => {
                let levels = &self.settings.format.discharging;
                let percent = self.state.percent
                    .context("no percent in state")?;

                match levels {
                    // levels is None, use default formatter
                    None => Some(&self.settings.format.default),
                    // levels is Some
                    Some(lvls) => Some(
                        find_current_level(lvls, &percent)?
                    )
                }
            }
        };
        
        Ok(template)
    }

    fn apply_strfmt_template(&self, template: &str) -> anyhow::Result<Option<String>> {
        apply_strfmt!(
            template,
            "p" => match self.state.percent {
                None => "N/A".to_string(),
                Some(v) => v.to_string(),
            },
            "t" => match self.state.time {
                None => "N/A".to_string(),
                Some(t) => {
                    let visual_time = &humantime::format_duration(t).to_string();
                    static RE: Lazy<Regex> = Lazy::new(
                        || Regex::new(r"\s[0-9]+s$").expect("could not build regex")
                    );
                    RE.replace(visual_time, "").to_string()
                }
            }
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

