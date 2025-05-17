use std::time;

use serde::Deserialize;
use smart_default::SmartDefault;

use crate::{impl_component_methods, new_from_value};

use super::Component;

// Custom command /////////////////////////////////////////////////////////////

#[derive(Debug, SmartDefault)]
pub struct Command {
    pub state: CommandState,
    pub settings: CommandSettings,
}

#[derive(Debug, SmartDefault)]
pub struct CommandState {
    pub last_updated: Option<time::Instant>,
    pub cache: Option<String>,
}

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CommandSettings {
    #[default(1000)]
    pub refresh_interval: u64,

    #[default(8)]
    pub signal: u32,

    #[default("echo -n ' hello world! '")]
    pub shell_command: String,
}

impl Component for Command {
    fn new_from_value(value: &serde_yml::Value) -> anyhow::Result<Self>
    where
        Self: std::marker::Sized,
    {
        new_from_value!(
            value => CommandSettings
        )
    }

    fn update_state(&mut self) -> anyhow::Result<()> {
        anyhow::bail!("not applicable")
    }

    fn get_strfmt_template(&self) -> anyhow::Result<Option<&str>> {
        anyhow::bail!("not applicable")
    }

    fn apply_strfmt_template(&self, _template: &str) -> anyhow::Result<Option<String>> {
        anyhow::bail!("not applicable")
    }

    fn set_cache(&mut self, _str: String) -> anyhow::Result<()> {
        anyhow::bail!("not applicable")
    }

    fn update(&mut self) -> anyhow::Result<()> {
        // run shell command
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&self.settings.shell_command)
            .output()?;

        if output.status.success() {
            // stdout to String
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            // Update cache
            self.state.cache = Some(stdout);
        } else {
            // stderr to String
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            // return io error
            Err(std::io::Error::new(std::io::ErrorKind::Other, stderr))?;
        }

        Ok(())
    }

    impl_component_methods!(
        get_last_updated,
        get_refresh_interval,
        get_signal_value,
        get_cache,
        default_output
    );
}
