// #![allow(dead_code)]
//   __                _     _        _
//  / _| ___ _ __ _ __(_)___| |_ __ _| |_ _   _ ___
// | |_ / _ \ '__| '__| / __| __/ _` | __| | | / __|
// |  _|  __/ |  | |  | \__ \ || (_| | |_| |_| \__ \
// |_|  \___|_|  |_|  |_|___/\__\__,_|\__|\__,_|___/
//

use std::{thread, time::Duration};

use args::Args;
use clap::Parser;
use config::Config;
use signals::signals_watch;

pub mod args;
pub mod components;
pub mod config;
pub mod signals;
pub mod utils;

// ideas:
// it's separated into blocks/modules
// each block has a type

// Alsa,
// Backlight,
// Battery,
// Time,

// Separator,
// Command,

// Cpu,
// Ram,
// Swap,
// CpuTemp,
// Wifi,
// Custom,

fn main() -> anyhow::Result<()> {
    // parse args
    let args = Args::parse();
    println!("Args: {:#?}", args);

    let config = Config::new(&args)?;
    println!("Config: {:#?}", config);

    println!("Hello, world!");

    signals_watch()?;

    for _ in 0..10 {
        println!("Hello, world!");
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn main() -> anyhow::Result<()> {
        let args = Args {
            config_path: "examples/config.yaml".into(),
        };

        let mut config = match Config::new(&args) {
            Ok(c) => c,
            Err(e) => {
                anyhow::bail!("crocdile crocodale -- {e}")
            },
        };
        // let mut config = Config::new(&args)
        //     .unwrap_or_else(|e| format!("failed to create config"));

        // printing
        for _ in 0..10 {
            println!(
                "Format:  {}",
                config
                .components
                .list
                .iter_mut()
                .map(|c| {
                    c.update().unwrap();
                    c.to_string()
                })
                .collect::<Vec<String>>()
                .join(&config.settings.default_separator)
            );

            thread::sleep(Duration::from_secs(1));
        }
        Ok(())
    }
}
