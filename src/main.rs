// #![allow(dead_code)]
//   __                _     _        _
//  / _| ___ _ __ _ __(_)___| |_ __ _| |_ _   _ ___
// | |_ / _ \ '__| '__| / __| __/ _` | __| | | / __|
// |  _|  __/ |  | |  | \__ \ || (_| | |_| |_| \__ \
// |_|  \___|_|  |_|  |_|___/\__\__,_|\__|\__,_|___/
//

use args::Args;
use clap::Parser;

pub mod args;
pub mod components;
pub mod config;
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

// #[derive(Debug, Parser)]
// #[command(author, version, about)]
// struct Args {
//     config: Option<String>,
// }

fn main() -> anyhow::Result<()> {
    // parse args
    let args = Args::parse();
    println!("Args: {:#?}", args);

    println!("Hello, world!");

    Ok(())
}
