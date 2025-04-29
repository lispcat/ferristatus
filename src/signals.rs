use anyhow::Context;
use libc::{SIGRTMAX, SIGRTMIN};
use signal_hook::iterator::Signals;
use std::thread;

pub fn signals_watch() -> anyhow::Result<()> {
    let rtmin = SIGRTMIN(); // 34
    let rtmax = SIGRTMAX(); // 64
    println!("min: {}, max: {}", rtmin, rtmax);

    for i in rtmin..=rtmax {
        let mut sig =
            Signals::new([i]).with_context(|| format!("Failed to define signal {}", i))?;

        thread::spawn(move || {
            for sig in sig.forever() {
                println!("Received signal RTMIN+{:?}", sig - rtmin);
            }
        });
    }

    Ok(())
}
