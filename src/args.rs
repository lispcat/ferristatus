use clap::Parser;
use smart_default::SmartDefault;
use std::path::PathBuf;

use crate::config::default_config_path;

#[derive(Debug, Parser, SmartDefault)]
#[command(author, version, about)]
pub struct Args {
    #[arg(
        short,
        long = "config",
        default_value_os_t = default_config_path(),
    )]
    /// Path to config file
    pub config_path: PathBuf,
}
