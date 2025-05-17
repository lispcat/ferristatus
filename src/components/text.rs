use serde::Deserialize;
use smart_default::SmartDefault;

use super::Component;

// Text ///////////////////////////////////////////////////////////////////////

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Text(String);

impl Component for Text {
    fn new_from_value(value: &serde_yml::Value) -> anyhow::Result<Self>
    where
        Self: std::marker::Sized,
    {
        {
            let text: Text = crate::deserialize_value!(value);
            Ok(text)
        }
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

    fn get_last_updated(&self) -> anyhow::Result<&Option<std::time::Instant>> {
        anyhow::bail!("not applicable")
    }

    fn get_refresh_interval(&self) -> anyhow::Result<&u64> {
        anyhow::bail!("not applicable")
    }

    fn get_signal_value(&self) -> anyhow::Result<Option<&u32>> {
        Ok(None)
    }

    fn get_cache(&self) -> anyhow::Result<Option<&str>> {
        Ok(Some(&self.0))
    }

    fn default_output(&self) -> anyhow::Result<&str> {
        anyhow::bail!("not applicable")
    }

    fn update(&mut self) -> anyhow::Result<()> {
        anyhow::bail!("not applicable")
    }

    fn update_check(&self) -> anyhow::Result<bool> {
        anyhow::bail!("not applicable")
    }

    fn update_maybe(&mut self) -> anyhow::Result<bool> {
        Ok(false)
    }
}
