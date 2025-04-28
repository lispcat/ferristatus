use crate::utils::default_config_path;
use clap::Parser;
use smart_default::SmartDefault;
use std::path::PathBuf;

#[derive(Debug, Parser, SmartDefault)]
#[command(author, version, about)]
pub struct Args {
    #[arg(
        short = 'c',
        long = "config",
        help = "Path to config file",
        default_value_os_t = default_config_path(),
        value_parser = clap::builder::PathBufValueParser::new()
    )]
    pub config_path: PathBuf,
}
