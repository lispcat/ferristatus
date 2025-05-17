use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use anyhow::Context;
use args::Args;
use clap::Parser;
use components::ComponentVecType;
use config::Config;

mod args;
mod components;
mod config;
mod signals;
mod utils;

fn update_check_all(components: &mut MutexGuard<'_, ComponentVecType>) -> anyhow::Result<()> {
    let components: &mut ComponentVecType = components;
    for c in components.iter_mut() {
        let mut lock = c.lock().expect("failed to lock");
        lock.update_maybe()?;
    }
    Ok(())
}

fn update_matching_signal(
    signal: u32,
    components: &mut MutexGuard<'_, ComponentVecType>,
) -> anyhow::Result<()> {
    for c in components.iter() {
        let mut c_guard = c.lock().expect("failed to lock");
        if c_guard.get_signal_value()? == Some(&signal) {
            c_guard.update()?;
            break;
        }
    }

    Ok(())
}

fn collect_all_cache_and_print(
    components: &MutexGuard<'_, ComponentVecType>,
) -> anyhow::Result<()> {
    for c in components.iter() {
        let c_guard = c.lock().expect("failed to lock");
        let cache = c_guard.get_cache()?;

        match cache {
            Some(s) => print!("{}", s),
            None => print!("(N/A: no cache)"),
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

    // get components
    let components: Arc<Mutex<ComponentVecType>> = Arc::new(Mutex::new(config.components.vec));
    let components_for_signal: Arc<Mutex<ComponentVecType>> = Arc::clone(&components);

    // start signal handler
    let signal_receiver = signals::signals_watch()?;

    // create signal watcher thread
    thread::spawn(move || -> anyhow::Result<()> {
        loop {
            // wait for signal
            if let Ok(signal) = signal_receiver.recv() {
                let mut components_guard: MutexGuard<'_, ComponentVecType> =
                    components_for_signal.lock().expect("failed to lock");

                // update only the corresponding component
                update_matching_signal(signal, &mut components_guard)?;

                // collect all and print
                collect_all_cache_and_print(&components_guard)?;
            }
        }
    });

    // run for 10 iterations
    for _ in 0..10 {
        {
            // Lock the components and cache_vec
            let mut components_guard: MutexGuard<'_, ComponentVecType> = components.lock().unwrap();

            // update check all
            update_check_all(&mut components_guard).context("failed to update all components")?;

            // collect all and print
            collect_all_cache_and_print(&components_guard)?;
        }
        thread::sleep(Duration::from_millis(config.settings.check_interval));
    }

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
        let components: Arc<Mutex<ComponentVecType>> = Arc::new(Mutex::new(config.components.vec));
        let components_for_signal: Arc<Mutex<ComponentVecType>> = Arc::clone(&components);

        // start signal handler
        let signal_receiver = signals::signals_watch()?;

        // create signal watcher thread
        thread::spawn(move || -> anyhow::Result<()> {
            loop {
                // wait for signal
                if let Ok(signal) = signal_receiver.recv() {
                    let mut components_guard: MutexGuard<'_, ComponentVecType> =
                        components_for_signal.lock().expect("failed to lock");

                    // update only the corresponding component
                    update_matching_signal(signal, &mut components_guard)?;

                    // collect all and print
                    collect_all_cache_and_print(&components_guard)?;
                }
            }
        });

        // run for 10 iterations
        for _ in 0..10 {
            {
                // Lock the components and cache_vec
                let mut components_guard: MutexGuard<'_, ComponentVecType> =
                    components.lock().unwrap();

                // update check all
                update_check_all(&mut components_guard)
                    .context("failed to update all components")?;

                // collect all and print
                collect_all_cache_and_print(&components_guard)?;
            }
            thread::sleep(Duration::from_millis(config.settings.check_interval));
        }

        Ok(())
    }
}
