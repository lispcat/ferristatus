use std::{thread, time::Duration};

use anyhow::Context;
use args::Args;
use clap::Parser;
use components::Component;
use config::Config;

mod args;
mod components;
mod config;

fn main() -> anyhow::Result<()> {
    // parse args
    let args = Args::parse();

    // load config
    let mut config = Config::new(&args)
        .map_err(|e| anyhow::anyhow!("crocdile crocodale -- {e}"))?;

    // run
    loop {
        // update components
        let val = config
            .components
            .vec
            .iter_mut()
            .map(|c| -> anyhow::Result<String> {
                c.update().context("component update failed")?;
                c.format().context("failed to format")
            })
            .collect::<Result<Vec<String>, _>>()?
            .join(&config.settings.default_separator);
        thread::sleep(Duration::from_millis(config.settings.check_interval));

        println!("{}", val);
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use crate::{args::Args, components::Component, config::Config};

    #[test]
    fn main() -> anyhow::Result<()> {
        let args = Args {
            config_path: "examples/config.yml".into(),
        };

        let mut config = Config::new(&args)
            .map_err(|e| anyhow::anyhow!("crocdile crocodale -- {e}"))?;

        for _ in 0..10 {
            println!(
                "Format:   {}",
                config
                    .components
                    .vec
                    .iter_mut()
                    .map(|c| {
                        c.update().expect("component update failed");
                        c.format().expect("failed to format")
                    })
                    .collect::<Vec<String>>()
                    .join(&config.settings.default_separator)
            );
            // TODO: make it only print when at least one component is actually updated
            thread::sleep(Duration::from_millis(config.settings.check_interval));
        }
        Ok(())
    }
}
