use std::{
    fs::File,
    io,
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use anyhow::Context;
use args::Args;
use components::ComponentVecType;
use config::Config;
use env_logger::{Builder, Env, Target};
use errors::MyErrors;
use log::LevelFilter;
use pidfile::PidFile;
use rand::Rng;
use signals::spawn_signal_responder_thread;

pub mod args;
pub mod components;
pub mod config;
pub mod errors;
pub mod signals;
pub mod utils;

/// Initialize logging support (to log file)
/// TODO: currently overwrites the file, so doesn't support multiple instances
/// (maybe use it in conjunction with the pid file by having same 6 digit ID?
/// and have it auto-dropped?)
pub fn init_logger() -> anyhow::Result<()> {
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
pub fn create_pid_file() -> anyhow::Result<PidFile> {
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
pub fn update_check_all(components: &mut MutexGuard<'_, ComponentVecType>) -> anyhow::Result<()> {
    for c in components.iter_mut() {
        c.lock()
            .map_err(MyErrors::from_poison_error)?
            .update_maybe()?;
    }

    Ok(())
}

/// Update components with a corresponding signal value.
pub fn update_matching_signal(
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
pub fn collect_all_cache(components: &MutexGuard<'_, ComponentVecType>) -> anyhow::Result<String> {
    let line: String = components
        .iter()
        .map(|c| -> anyhow::Result<String> {
            let c_guard: MutexGuard<_> = c.lock().map_err(MyErrors::from_poison_error)?;

            Ok(c_guard
                .get_cache()?
                .unwrap_or("(N/A: no cache)")
                .to_string())
        })
        .collect::<anyhow::Result<_>>()?;

    Ok(line)
}

pub fn update_and_print(components: &Arc<Mutex<ComponentVecType>>) -> anyhow::Result<()> {
    // lock the components
    let mut components_guard: MutexGuard<'_, ComponentVecType> =
        components.lock().map_err(MyErrors::from_poison_error)?;

    // update check all
    update_check_all(&mut components_guard).context("failed to update all components")?;

    // collect all and print
    println!("{}", collect_all_cache(&components_guard)?);

    Ok(())
}

macro_rules! sleep_for_duration {
    ($interval:expr) => {
        thread::sleep(Duration::from_millis($interval))
    };
}

/// The main body of the program.
pub fn run_program(args: Args, max_iter: Option<u32>) -> anyhow::Result<()> {
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
    spawn_signal_responder_thread(components_for_signal)?;

    // run until terminated
    match max_iter {
        None => loop {
            update_and_print(&components)?;
            sleep_for_duration!(config.settings.check_interval);
        },
        Some(n) => {
            for _ in 0..=n {
                update_and_print(&components)?;
                sleep_for_duration!(config.settings.check_interval);
            }
        }
    };

    Ok(())
}
