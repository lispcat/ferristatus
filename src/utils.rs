use anyhow::Context;
use itertools::Itertools;

pub fn sort_levels(levels: &mut Option<Vec<(i32, String)>>) {
    levels.as_ref().map(|lvls| {
        lvls.clone()
            .into_iter()
            .sorted_by(|a, b| a.0.cmp(&b.0))
            .collect::<Vec<(i32, String)>>()
    });
}

pub fn find_current_level<'a>(
    levels: &'a [(i32, String)],
    current: &i32,
) -> anyhow::Result<&'a str> {
    levels
        .iter()
        .find_or_last(|(ceiling, _)| current > ceiling)
        .map(|(_, format_str)| format_str.as_str())
        .context("failed to find_current_level")
    // println!("DEBUG: CURR: {}", current);
    // for pair in levels.iter() {
    //     let (ceiling, format_str) = pair;
    //     println!("DEBUG: {}, {}", ceiling, format_str);
    //     if current > ceiling {
    //         println!("DEBUG: FOUND: {}", ceiling);
    //         return Ok(format_str);
    //     }
    // }
    // Err(anyhow::anyhow!("failed to find current level"))
}

#[macro_export]
macro_rules! deserialize_value {
    ( $value:tt ) => {{
        serde_yml::from_value($value.clone())?
    }};
}

#[macro_export]
macro_rules! new_from_value {
    ( $value:tt => $component_settings:ident, sort_levels: $sort:expr ) => {{
        let mut settings: $component_settings = $crate::deserialize_value!($value);

        if $sort {
            $crate::utils::sort_levels(&mut settings.format.levels);
        }

        Ok(Self {
            settings,
            ..Self::default()
        })
    }};
    ( $value:tt => $component_settings:ident ) => {{
        let settings: $component_settings = $crate::deserialize_value!($value);

        Ok(Self {
            settings,
            ..Self::default()
        })
    }};
}

#[macro_export]
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

    (get_signal_value) => {
        fn get_signal_value(&self) -> anyhow::Result<Option<&u32>> {
            Ok(Some(&self.settings.signal))
        }
    };

    (get_cache) => {
        fn get_cache(&self) -> anyhow::Result<Option<&str>> {
            Ok(self.state.cache.as_ref().map(|x| x.as_str()))
        }
    };

    (default_output) => {
        fn default_output(&self) -> anyhow::Result<&str> {
            Ok("N/A: (default_output)")
        }
    };

    // recursion to support multiple args
    ($first:ident, $($rest:ident),+) => {
        impl_component_methods!($first);
        impl_component_methods!($($rest),+);
    };
}

#[macro_export]
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
