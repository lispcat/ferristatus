use serde::Deserialize;
use std::fmt::{Debug, Display};

pub trait Component: Debug + Display {
    fn name(&self) -> String;
    fn update(&mut self) -> anyhow::Result<()>;
}

pub trait ComponentSettings: Debug + for<'a> Deserialize<'a> {}

pub trait ComponentState: Debug {}

pub trait ComponentFormat: Debug {}
