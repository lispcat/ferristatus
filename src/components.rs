use serde::Deserialize;
use std::fmt;

pub mod alsa;
pub mod backlight;
pub mod battery;
pub mod component_list;
pub mod time;

pub trait Component: fmt::Debug {
    fn update(&mut self) -> anyhow::Result<()>;
}

pub trait ComponentSettings: fmt::Debug + for<'a> Deserialize<'a> {}
