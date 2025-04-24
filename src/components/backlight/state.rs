use smart_default::SmartDefault;
use std::time;

use crate::components::ComponentState;

#[derive(Debug, SmartDefault)]
pub struct BacklightState {
    pub perc: Option<i32>,
    pub last_updated: Option<time::Instant>,
}

impl ComponentState for BacklightState {}
