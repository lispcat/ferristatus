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
    fn main() {
        let mut config = Config::new(&Args {
            config_path: "tests/config.json".into(),
            ..Args::default()
        })
        .expect("failed to get config");

        println!("DEBUG: {:#?}", config.components.list);

        for c in config.components.list.iter_mut() {
            c.update().unwrap()
        }

        // printing
        for _ in 0..10 {
            println!("> updating...");
            println!(
                "Format:  {}",
                config
                    .components
                    .list
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join(&config.settings.default_separator)
            );

            thread::sleep(Duration::from_secs(1));
        }
    }
}
