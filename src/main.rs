use std::{thread, time::Duration};

use anyhow::Context;
use args::Args;
use clap::Parser;
use components::Component;
use config::Config;

mod args;
mod components;
mod config;

fn main() -> anyhow::Result<()> {
    // parse args
    let args = Args::parse();

    // load config
    let config = Config::new(&args)
        .context("failed to create config")?;

    // run
    let mut components = config.components.vec;
    loop {
        // // update components
        // let val = components
        //     .iter_mut()
        //     .map(|c| -> anyhow::Result<String> {
        //         let _ = c.update_maybe()?;
        //         // if c.update_check()? {
        //         //     c.update().context("component update failed")?;
        //         //     c.format().context("failed to format")
        //         // } else {
        //         //     match c.get_format_cache().context("failed to get format cache")? {
        //         //         Some(s) => Ok(s),
        //         //         None => Err(anyhow::anyhow!("get_format_cache returned None")),
        //         //     }
        //         // }
        //     })
        //     .collect::<Result<Vec<String>, _>>()?
        //     .join(&config.settings.default_separator);
        for c in components.iter_mut() {
            let _ = c.update_maybe()?;
            print!("{}", c);
        }
        println!();
        thread::sleep(Duration::from_millis(config.settings.check_interval));

        // println!("{}", val);
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use anyhow::Context;

    use crate::{args::Args, config::Config};

    #[test]
    fn main() -> anyhow::Result<()> {
        let args = Args {
            config_path: "examples/config.new.yml".into(),
        };

        let config = Config::new(&args)
            .context("failed to create config")?;

        // run
        let mut components = config.components.vec;

        // run for 10 iterations
        for _ in 0..10 {
            // update all components
            for c in components.iter_mut() {
                c.update_maybe()?;
            }

            // get output for all components
            let output = components
                .iter()
                .map(|c| -> anyhow::Result<&Option<String>> {
                    c.get_cache()
                })
                .collect::<Result<Vec<&Option<String>>, _>>()?;

            // print all
            for c in output {
                match c {
                    Some(v) => {
                        print!("{}", v);
                    },
                    None => print!("N/A(no_cache)"),
                }
                println!();
            }
            //     // update components
            //     let val = components
            //         .iter_mut()
            //         .map(|c| -> anyhow::Result<String> {
            //             if c.update_check()? {
            //                 c.update().context("component update failed")?;
            //                 c.format().context("failed to format")
            //             } else {
            //                 match c.get_format_cache().context("failed to get format cache")? {
            //                     Some(s) => Ok(s),
            //                     None => Err(anyhow::anyhow!("get_format_cache returned None")),
            //                 }
            //             }
            //         })
            //         .collect::<Result<Vec<String>, _>>()?
            //         .join(&config.settings.default_separator);
            //     thread::sleep(Duration::from_millis(config.settings.check_interval));

            //     println!("{}", val);
            // for c in components.iter_mut() {
            //     let _ = c.update_maybe()?;
            //     print!("{}", c);
            // }
            // println!();
            thread::sleep(Duration::from_millis(config.settings.check_interval));
        }

        Ok(())
    }
}
