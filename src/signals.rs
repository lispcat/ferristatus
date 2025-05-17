use anyhow::Context;
use libc::{SIGRTMAX, SIGRTMIN};
use signal_hook::iterator::Signals;
use std::{
    sync::mpsc::{self, Receiver},
    thread,
};

pub fn signals_watch() -> anyhow::Result<Receiver<u32>> {
    let rtmin = SIGRTMIN(); // 34
    let rtmax = SIGRTMAX(); // 64
    eprintln!("LOG: RTMIN: {}, RTMAX: {}", rtmin, rtmax);

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
