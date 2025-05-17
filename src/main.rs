use std::{
    io,
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use anyhow::Context;
use args::Args;
use clap::Parser;
use components::ComponentVecType;
use config::Config;
use pidfile::PidFile;
use rand::Rng;

mod args;
mod components;
mod config;
mod signals;
mod utils;

/// Creates a pid file with format /tmp/ferristatus-XXXXXX.pid
fn create_pid_file() -> anyhow::Result<PidFile> {
    match PidFile::new(format!("/tmp/ferristatus-{}.pid", {
        let mut rng = rand::rng();
        (0..6)
            .map(|_| rng.random_range(0..10).to_string())
            .collect::<String>()
    })) {
        Ok(v) => Ok(v),
        Err(e) => match e.kind() {
            io::ErrorKind::AddrInUse => Ok(create_pid_file()?),
            _ => Err(e.into()),
        },
    }
}

/// Update every component as needed.
fn update_check_all(components: &mut MutexGuard<'_, ComponentVecType>) -> anyhow::Result<()> {
    let components: &mut ComponentVecType = components;
    for c in components.iter_mut() {
        let mut lock = c.lock().expect("failed to lock");
        lock.update_maybe()?;
    }
    Ok(())
}

/// Update the first component with a corresponding signal value.
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

/// Collect the cache from every component and print it to stdout.
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

/// The main body of the program.
fn run_program(args: Args, testing: bool) -> anyhow::Result<()> {
    // parse config
    let config = Config::new(&args).context("failed to create config")?;

    // get components
    let components: Arc<Mutex<ComponentVecType>> = Arc::new(Mutex::new(config.components.vec));
    let components_for_signal: Arc<Mutex<ComponentVecType>> = Arc::clone(&components);

    // create pid file
    let _pidfile = create_pid_file()?;

    // create signal watcher thread
    thread::spawn(move || -> anyhow::Result<()> {
        // start signal handler
        let signal_receiver = signals::signals_watch()?;
        // start signal handling loop
        loop {
            // wait for signal
            if let Ok(signal) = signal_receiver.recv() {
                // lock the components
                let mut components_guard: MutexGuard<'_, ComponentVecType> =
                    components_for_signal.lock().expect("failed to lock");
                // update only the corresponding component
                update_matching_signal(signal, &mut components_guard)?;
                // collect all and print
                collect_all_cache_and_print(&components_guard)?;
            }
        }
    });

    // run until terminated
    let mut num_iterations = 0;
    loop {
        {
            // lock the components
            let mut components_guard: MutexGuard<'_, ComponentVecType> =
                components.lock().expect("failed to lock");
            // update check all
            update_check_all(&mut components_guard).context("failed to update all components")?;
            // collect all and print
            collect_all_cache_and_print(&components_guard)?;
        }
        thread::sleep(Duration::from_millis(config.settings.check_interval));

        if testing {
            num_iterations += 1;
            if num_iterations > 10 {
                return Ok(());
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    // parse args
    let args = Args::parse();
    run_program(args, false)
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
        super::run_program(args, true)
    }
}
