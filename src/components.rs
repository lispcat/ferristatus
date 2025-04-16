use serde::Deserialize;
use std::fmt::{Debug, Display};

pub mod alsa;
pub mod backlight;
pub mod battery;
pub mod component_list;
pub mod time;

pub trait Component: Debug + Display {
    fn update(&mut self) -> anyhow::Result<()>;
}

pub trait ComponentSettings: Debug + for<'a> Deserialize<'a> {}
