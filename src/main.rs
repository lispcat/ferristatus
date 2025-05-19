use clap::Parser;
use ferristatus::{args::Args, run_program};

fn main() -> anyhow::Result<()> {
    // parse args
    let args = Args::parse();
    run_program(args, None)
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
        super::run_program(args, Some(2))
    }
}
