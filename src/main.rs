// #![allow(dead_code)]
//   __                _     _        _
//  / _| ___ _ __ _ __(_)___| |_ __ _| |_ _   _ ___
// | |_ / _ \ '__| '__| / __| __/ _` | __| | | / __|
// |  _|  __/ |  | |  | \__ \ || (_| | |_| |_| \__ \
// |_|  \___|_|  |_|  |_|___/\__\__,_|\__|\__,_|___/
//

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
        for c in config.components.list.iter_mut() {
            c.update().unwrap();
        }
        println!("Config: {:#?}", config.components.list);
    }
}
