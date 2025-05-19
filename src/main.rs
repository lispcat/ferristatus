use std::{
    fs::File,
    io,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    thread,
    time::Duration,
};

use anyhow::Context;
use args::Args;
use clap::Parser;
use components::ComponentVecType;
use config::Config;
use env_logger::{Builder, Env, Target};
use log::LevelFilter;
use pidfile::PidFile;
use rand::Rng;

mod args;
mod components;
mod config;
mod signals;
mod utils;

/// Custom error type for this crate
#[derive(thiserror::Error, Debug)]
enum MyErrors {
    #[error("Failed to lock mutex: {0}")]
    MutexLockError(String),
}

impl MyErrors {
    fn from_poison_error<T>(e: PoisonError<T>) -> Self {
        MyErrors::MutexLockError(e.to_string())
    }
}

/// Initialize logging support (to log file)
/// TODO: currently overwrites the file, so doesn't support multiple instances
/// (maybe use it in conjunction with the pid file by having same 6 digit ID?
/// and have it auto-dropped?)
fn init_logger() -> anyhow::Result<()> {
    let path = "/tmp/ferristatus.log";
    let file = File::create(path).expect("failed to create log file"); // TODO: dont overwrite pre-existing
    let mut builder = Builder::from_env(Env::default());
    builder
        .target(Target::Pipe(Box::new(file)))
        .filter_level(LevelFilter::Info)
        .init();
    Ok(())
}

/// Creates a pid file with format /tmp/ferristatus-XXXXXX.pid
/// TODO: double-check and make sure that the PID file is being auto-deleted.
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
    for c in components.iter_mut() {
        c.lock()
            .map_err(MyErrors::from_poison_error)?
            .update_maybe()?;
    }

    Ok(())
}

/// Update components with a corresponding signal value.
fn update_matching_signal(
    signal: u32,
    components: &mut MutexGuard<'_, ComponentVecType>,
) -> anyhow::Result<()> {
    for c in components.iter() {
        let mut c_guard: MutexGuard<_> = c.lock().map_err(MyErrors::from_poison_error)?;

        if c_guard.get_signal_value()? == Some(&signal) {
            c_guard.update()?;
            continue;
        }
    }

    Ok(())
}

/// Collect the cache from every component and print it to stdout.
fn collect_all_cache_and_print(
    components: &MutexGuard<'_, ComponentVecType>,
) -> anyhow::Result<()> {
    for c in components.iter() {
        let c_guard: MutexGuard<_> = c.lock().map_err(MyErrors::from_poison_error)?;

        match c_guard.get_cache()? {
            Some(s) => print!("{}", s),
            None => print!("(N/A: no cache)"),
        }
    }
    println!();

    Ok(())
}

/// The main body of the program.
fn run_program(args: Args, testing: bool) -> anyhow::Result<()> {
    // set up logging
    init_logger()?;

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
                // logging
                log::info!("received RT signal: {}", signal);

                // lock the components
                let mut components_guard: MutexGuard<'_, ComponentVecType> = components_for_signal
                    .lock()
                    .map_err(MyErrors::from_poison_error)?;
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
                components.lock().map_err(MyErrors::from_poison_error)?;
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
