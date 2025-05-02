use serde::Deserialize;
use smart_default::SmartDefault;

use super::Component;

// Text ///////////////////////////////////////////////////////////////////////

#[derive(Debug, SmartDefault, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Text(String);

impl Component for Text {
    fn name(&self) -> String {
        "text".to_string()
    }

    fn get_refresh_interval(&self) -> u32 {
        0
    }

    fn get_last_updated(&self) -> Option<std::time::Instant> {
        None
    }

    fn update(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_format_str(&self) -> anyhow::Result<String> {
        Ok(self.0.clone())
    }

    fn format(&mut self) -> anyhow::Result<String> {
        self.get_format_str()
    }

    fn update_format_cache(&mut self, _str: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_format_cache(&self) -> anyhow::Result<Option<String>> {
        Ok(None)
    }
}
