use std::{env, path::PathBuf};

use crate::{args::Args, config::Config};

pub fn default_config_path() -> PathBuf {
    let config_dir = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|p| p.join(".config"))
                .expect("Cannot find HOME directory")
        });

    config_dir.join("ferristatus").join("config.json")
}

pub fn parse_test_config() -> Config {
    let args = Args {
        config_path: "tests/config.json".into(),
        ..Args::default()
    };

    Config::new(&args).expect("failed to get config")
}

/// Implement a new() struct for a Type using $type::default()
#[macro_export]
macro_rules! impl_default_new {
    ($type:ident) => {
        impl $type {
            pub fn new() -> $type {
                $type::default()
            }
        }
    };
}
