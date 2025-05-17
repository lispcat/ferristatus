use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use anyhow::Context;
use args::Args;
use clap::Parser;
use components::Component;
use config::Config;

mod args;
mod components;
mod config;
mod signals;
mod utils;

fn update_check_all(
    components: &mut MutexGuard<'_, Vec<Box<dyn Component>>>,
) -> anyhow::Result<()> {
    let components: &mut Vec<Box<dyn Component>> = components;
    for c in components.iter_mut() {
        c.update_maybe()?;
    }
    Ok(())
}

fn collect_cache_from_components<'a>(
    components: &'a MutexGuard<'a, Vec<Box<dyn Component>>>,
) -> anyhow::Result<Vec<Option<&'a str>>> {
    components
        .iter()
        .map(|c| -> anyhow::Result<Option<&str>> { c.get_cache() })
        .collect::<Result<Vec<_>, _>>()
}

fn print_collected_cache(cache_vec: &Vec<Option<&str>>) -> anyhow::Result<()> {
    for c in cache_vec.iter() {
        match c {
            Some(v) => print!("{}", v),
            None => print!("N/A: (no_cache)"),
        }
    }
    println!();
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // parse args
    let args = Args::parse();

    // parse config
    let config = Config::new(&args).context("failed to create config")?;

    // get config
    let mut components = config.components.vec;

    // start signal handler
    let signal_receiver = signals::signals_watch()?;

    // run until killed
    // loop {
    //     update_all_components(&mut components).context("failed to update component")?;

    //     let output = collect_cache_for_components(&components)
    //         .context("failed to collect component cache")?;

    //     print_collected_cache(&output).context("failed to print collected component cache")?;

    //     // sleep or signal interrupt
    //     if let Some(signal) = signals::wait_for_signal_or_timeout(
    //         &signal_receiver,
    //         Duration::from_millis(config.settings.check_interval),
    //     )? {
    //         println!("WWWWWW: SIGNAL RECEIVED: {}", signal);
    //     }
    // }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        ops::Deref,
        sync::{Arc, Mutex},
    };

    use crate::signals::wait_for_signal;

    use super::*;

    #[test]
    fn main() -> anyhow::Result<()> {
        // parse args
        let args = Args {
            config_path: "examples/config.yml".into(),
        };

        // parse config
        let config = Config::new(&args).context("failed to create config")?;

        // get components
        let mut components = Arc::new(Mutex::new(config.components.vec));

        // start signal handler
        let signal_receiver = signals::signals_watch()?;

        // create signal watcher thread
        thread::spawn(move || loop {
            // // wait for signal
            // if let Ok(signal) = signal_receiver.recv() {
            //     println!("DEBUG: update appropriate component!");

            //     // update only the corresponding component
            //     let mut components_guard = components.lock().unwrap();

            //     for c in components_guard.iter() {
            //         println!("DEBUG: wah: {:#?}", c);
            //     }

            //     // print collection
            // }
        });

        // run for 10 iterations
        for _ in 0..10 {
            // Lock the components and cache_vec
            let mut components_guard = components.lock().unwrap();

            // update all
            update_check_all(&mut components_guard).context("failed to update all components")?;

            // save all to cache_vec
            let cache_vec = collect_cache_from_components(&components_guard)
                .context("failed to collect cache from components")?;

            // print all
            print_collected_cache(&cache_vec)
                .context("failed to print collected components cache")?;

            thread::sleep(Duration::from_millis(config.settings.check_interval));
        }

        Ok(())
    }
}
