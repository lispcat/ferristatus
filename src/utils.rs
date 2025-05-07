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

#[macro_export]
macro_rules! deserialize_value {
    ( $value:tt ) => {{
        serde_yml::from_value($value.clone())?
    }};
}

#[macro_export]
macro_rules! new_self_from_settings {
    ( $settings:tt ) => {{
        Self {
            $settings,
            ..Self::default()
        }
    }};
}

