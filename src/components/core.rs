use serde::Deserialize;
use std::fmt::{Debug, Display};

pub trait Component: Debug + Display {
    fn name(&self) -> String;
    fn update(&mut self) -> anyhow::Result<()>;
    fn get_format_string(&self) -> String;
    fn eval_strfmt(&self, format_str: &str) -> anyhow::Result<String>;
}

pub trait ComponentSettings: Debug + for<'a> Deserialize<'a> {
}

pub trait ComponentState: Debug {}

pub trait ComponentFormat: Debug {}
