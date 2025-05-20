use anyhow::Context;
use libc::{SIGRTMAX, SIGRTMIN};
use signal_hook::iterator::Signals;
use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex, MutexGuard,
    },
    thread,
};

use crate::{
    collect_all_cache, components::ComponentVecType, errors::MyErrors, update_matching_signal,
};

pub fn signals_watch() -> anyhow::Result<Receiver<u32>> {
    let rtmin = SIGRTMIN(); // 34
    let rtmax = SIGRTMAX(); // 64
    log::info!("RTMIN: {}, RTMAX: {}", rtmin, rtmax);

    let (tx, rx) = mpsc::channel();

    for i in rtmin..=rtmax {
        let mut sig =
            Signals::new([i]).with_context(|| format!("Failed to define signal {}", i))?;

        // clone sender for each thread
        let thread_tx = tx.clone();

        thread::spawn(move || {
            for sig in sig.forever() {
                // Send the signal number through the channel
                let _ = thread_tx.send((sig - rtmin) as u32); // We don't care if the send fails
            }
        });
    }

    // return the receiver
    Ok(rx)
}

pub fn spawn_signal_responder_thread(
    components_for_thread: Arc<Mutex<ComponentVecType>>,
) -> anyhow::Result<()> {
    // create signal watcher thread
    thread::spawn(move || -> anyhow::Result<()> {
        // start signal handler
        let signal_receiver = signals_watch()?;
        // start signal handling loop
        loop {
            // wait for signal
            if let Ok(signal) = signal_receiver.recv() {
                // logging
                log::info!("received RT signal: {}", signal);

                // lock the components
                let mut components_guard: MutexGuard<'_, ComponentVecType> = components_for_thread
                    .lock()
                    .map_err(MyErrors::from_poison_error)?;

                // update only the corresponding component
                update_matching_signal(signal, &mut components_guard)?;

                // collect all and print
                println!("{}", collect_all_cache(&components_guard)?);
            }
        }
    });

    Ok(())
}
