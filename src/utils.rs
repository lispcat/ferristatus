use anyhow::Context;
use itertools::Itertools;

pub fn sort_levels(levels: &mut Option<Vec<(i32, String)>>) {
    levels
        .as_ref()
        .map(|lvls| {
            lvls.clone()
                .into_iter()
                .sorted_by(|a, b| a.0.cmp(&b.0))
                .collect::<Vec<(i32, String)>>()
        });
}

pub fn find_current_level<'a, T: PartialOrd<i32>>(
    levels: &'a [(i32, String)],
    current: &T
) -> anyhow::Result<&'a str> {
    levels
        .iter()
        .find(|(ceiling, _)| current <= ceiling)
        .map(|(_, format_str)| format_str.as_str())
        .context("(N/A: could not find level)")
}

macro_rules! deserialize_value {
    ( $value:tt ) => {{
        serde_yml::from_value($value.clone())?
    }};
}
pub(crate) use deserialize_value;

macro_rules! new_from_value {
    ( $value:tt => $component_settings:ident, sort_levels: $sort:expr ) => {{
        let mut settings: $component_settings = deserialize_value!($value);

        if $sort {
            crate::utils::sort_levels(&mut settings.format.levels);
        }

        Ok(Self {
            settings,
            ..Self::default()
        })
    }};
}
pub(crate) use new_from_value;

macro_rules! impl_component_methods {
    (set_cache) => {
        fn set_cache(&mut self, str: String) -> anyhow::Result<()> {
            self.state.cache = Some(str);
            Ok(())
        }
    };

    (get_last_updated) => {
        fn get_last_updated(&self) -> anyhow::Result<&Option<std::time::Instant>> {
            Ok(&self.state.last_updated)
        }
    };

    (get_refresh_interval) => {
        fn get_refresh_interval(&self) -> anyhow::Result<&u64> {
            Ok(&self.settings.refresh_interval)
        }
    };

    (get_cache) => {
        fn get_cache(&self) -> anyhow::Result<&Option<String>> {
            Ok(&self.state.cache)
        }
    };

    (default_output) => {
        fn default_output(&self) -> anyhow::Result<&str> {
            Ok("N/A(default_output)")
        }
    };

    // recursion to support multiple args
    ($first:ident, $($rest:ident),+) => {
        impl_component_methods!($first);
        impl_component_methods!($($rest),+);
    };
}
pub(crate) use impl_component_methods;

macro_rules! apply_strfmt {
    ( $template:expr, $($key:expr => $value:expr),* $(,)? ) => {{
        let vars: std::collections::HashMap<String, String> = std::collections::HashMap::from([
            $(
                ($key.to_owned(), $value),
            )*
        ]);
        Ok(Some(strfmt::strfmt($template, &vars)?))
    }};
}
pub(crate) use apply_strfmt;
