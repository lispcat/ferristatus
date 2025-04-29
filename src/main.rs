use args::Args;
use clap::Parser;

mod args;
mod components;
mod config;

fn main() {
    // parse args
    let args = Args::parse();

    // load config

    // run
    for _ in 0..10 {
        println!("wah");
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use crate::{args::Args, components::Component, config::Config};

    #[test]
    fn main() -> anyhow::Result<()> {
        let args = Args {
            config_path: "examples/config.yml".into(),
        };

        let mut config = match Config::new(&args) {
            Ok(c) => c,
            Err(e) => anyhow::bail!("crocdile crocodale -- {e}"),
        };

        for _ in 0..10 {
            println!(
                "Format:   {}",
                config
                    .components
                    .vec
                    .iter_mut()
                    .map(|c| {
                        c.update().expect("component update failed");
                        c.format().expect("failed to format")
                    })
                    .collect::<Vec<String>>()
                    .join(&config.settings.default_separator)
            );
            thread::sleep(Duration::from_secs(1));
        }
        Ok(())
    }
}
