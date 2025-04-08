use libc::{SIGRTMAX, SIGRTMIN};
use signal_hook::iterator::Signals;
use std::{error::Error, thread};

pub fn signals_watch() -> Result<(), Box<dyn Error>> {
    let rtmin = SIGRTMIN(); // 34
    let rtmax = SIGRTMAX(); // 64
    println!("min: {}, max: {}", rtmin, rtmax);

    for i in rtmin..=rtmax {
        let mut sig =
            Signals::new([i]).map_err(|e| format!("Failed to define signal {}: {}", i, e))?;

        thread::spawn(move || {
            for sig in sig.forever() {
                println!("Received signal RTMIN+{:?}", sig - rtmin);
            }
        });
    }

    Ok(())
}
