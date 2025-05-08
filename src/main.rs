use std::{thread, time::Duration};

use anyhow::Context;
use args::Args;
use clap::Parser;
use components::Component;
use config::Config;

mod args;
mod components;
mod config;
mod utils;

fn update_all_components(components: &mut Vec<Box<dyn Component>>) -> anyhow::Result<()> {
    for c in components.iter_mut() {
        c.update_maybe()?;
    }
    Ok(())
}

fn collect_cache_for_components(
    components: &Vec<Box<dyn Component>>
) -> anyhow::Result<Vec<&Option<String>>> {
    components
        .iter()
        .map(|c| -> anyhow::Result<&Option<String>> {
            c.get_cache()
        })
        .collect::<Result<Vec<_>, _>>()
}

fn print_collected_cache(cache_collected: &Vec<&Option<String>>) -> anyhow::Result<()> {
    for c in cache_collected {
        match c {
            Some(v) => {
                print!("{}", v);
            },
            None => print!("N/A(no_cache)"),
        }
    }
    println!();
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // parse args
    let args = Args::parse();

    // parse config
    let config = Config::new(&args)
        .context("failed to create config")?;

    // get config
    let mut components = config.components.vec;

    // run until killed
    loop {

        update_all_components(&mut components)
            .context("failed to update component")?;

        let output = collect_cache_for_components(&components)
            .context("failed to collect component cache")?;

        print_collected_cache(&output)
            .context("failed to print collected component cache")?;

        // Pause
        thread::sleep(Duration::from_millis(config.settings.check_interval));
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main() -> anyhow::Result<()> {
        // parse args
        let args = Args {
            config_path: "examples/config.new.yml".into(),
        };

        // parse config
        let config = Config::new(&args)
            .context("failed to create config")?;

        // get components
        let mut components = config.components.vec;

        // run for 10 iterations
        for _ in 0..10 {

            update_all_components(&mut components)
                .context("failed to update component")?;

            let output = collect_cache_for_components(&components)
                .context("failed to collect component cache")?;

            print_collected_cache(&output)
                .context("failed to print collected component cache")?;

            // Pause
            thread::sleep(Duration::from_millis(config.settings.check_interval));
        }

        Ok(())
    }
}
