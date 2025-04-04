use std::fmt::{self, Display};

use serde::Deserialize;

pub mod alsa;
pub mod backlight;
pub mod battery;
pub mod time;
pub mod utils;

pub trait Component: fmt::Debug {}

impl Component for alsa::Alsa {}
impl Component for backlight::Backlight {}
impl Component for battery::Battery {}
impl Component for time::Time {}

pub trait ComponentSettings: fmt::Debug + for<'a> Deserialize<'a> {}

impl ComponentSettings for alsa::AlsaSettings {}
impl ComponentSettings for backlight::BacklightSettings {}
impl ComponentSettings for battery::BatterySettings {}
impl ComponentSettings for time::TimeSettings {}
