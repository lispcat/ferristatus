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
    components: &mut MutexGuard<'_, Vec<Arc<Mutex<dyn Component + Send + Sync>>>>,
) -> anyhow::Result<()> {
    let components: &mut Vec<Arc<Mutex<dyn Component + Send + Sync>>> = components;
    for c in components.iter_mut() {
        let mut lock = c.lock().expect("failed to lock");
        lock.update_maybe()?;
    }
    Ok(())
}

// TODO: consider printing in here? (just because i have to clone a vec of strings)
fn collect_cache_from_components<'a>(
    components: &'a MutexGuard<'a, Vec<Arc<Mutex<dyn Component + Send + Sync>>>>,
) -> anyhow::Result<Vec<Option<String>>> {
    components
        .iter()
        .map(|c| -> anyhow::Result<Option<String>> {
            let lock = c.lock().expect("failed to lock");
            let cache = lock.get_cache()?.context("TOFIX: expected Some")?;
            Ok(Some(cache.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()
}

fn print_collected_cache(cache_vec: &Vec<Option<String>>) -> anyhow::Result<()> {
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
        let components = Arc::new(Mutex::new(config.components.vec));
        let components_for_signal = Arc::clone(&components);

        // start signal handler
        let signal_receiver = signals::signals_watch()?;

        // create signal watcher thread
        thread::spawn(move || -> anyhow::Result<()> {
            loop {
                // wait for signal
                if let Ok(signal) = signal_receiver.recv() {
                    let components_guard = components_for_signal.lock().expect("failed to lock");

                    // update only the corresponding component
                    for c in components_guard.iter() {
                        let mut c_guard = c.lock().expect("failed to lock");
                        if c_guard.get_signal_value()? == Some(&signal) {
                            c_guard.update()?;
                            break;
                        }
                    }

                    // collect
                    let cache_vec = collect_cache_from_components(&components_guard)
                        .context("failed to collect cache from components")?;

                    // print
                    print_collected_cache(&cache_vec)
                        .context("failed to print collected components cache")?;
                }
            }
        });

        // run for 10 iterations
        for _ in 0..10 {
            {
                // Lock the components and cache_vec
                let mut components_guard = components.lock().unwrap();

                // update all
                update_check_all(&mut components_guard)
                    .context("failed to update all components")?;

                // save all to cache_vec
                let cache_vec = collect_cache_from_components(&components_guard)
                    .context("failed to collect cache from components")?;

                // print all
                print_collected_cache(&cache_vec)
                    .context("failed to print collected components cache")?;
            }
            thread::sleep(Duration::from_millis(config.settings.check_interval));
        }

        Ok(())
    }
}
