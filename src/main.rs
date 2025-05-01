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
    let config = Config::new(&args)
        .context("failed to create config")?;

    // run
    let mut components = config.components.vec;
    loop {
        // update components
        let val = components
            .iter_mut()
            .map(|c| -> anyhow::Result<String> {
                if c.update_check()? {
                    c.update().context("component update failed")?;
                    c.format().context("failed to format")
                } else {
                    match c.get_format_cache().context("failed to get format cache")? {
                        Some(s) => Ok(s),
                        None => Err(anyhow::anyhow!("get_format_cache returned None")),
                    }
                }
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

    use anyhow::Context;

    use crate::{args::Args, components::Component, config::Config};

    #[test]
    fn main() -> anyhow::Result<()> {
        let args = Args {
            config_path: "examples/config.yml".into(),
        };

        let config = Config::new(&args)
            .context("failed to create config")?;

        // run
        let mut components = config.components.vec;
        for _ in 0..10 {
            // update components
            let val = components
                .iter_mut()
                .map(|c| -> anyhow::Result<String> {
                    if c.update_check()? {
                        c.update().context("component update failed")?;
                        c.format().context("failed to format")
                    } else {
                        match c.get_format_cache().context("failed to get format cache")? {
                            Some(s) => Ok(s),
                            None => Err(anyhow::anyhow!("get_format_cache returned None")),
                        }
                    }
                })
                .collect::<Result<Vec<String>, _>>()?
                .join(&config.settings.default_separator);
            thread::sleep(Duration::from_millis(config.settings.check_interval));

            println!("{}", val);
        }

        Ok(())
    }
}
