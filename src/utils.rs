use std::{collections::HashMap, env, fmt::Display, path::PathBuf, str::FromStr};

use serde::{Deserialize, Deserializer};
use std::hash::Hash;
use strfmt::{strfmt, DisplayStr};

use crate::{args::Args, config::Config};

pub fn default_config_path() -> PathBuf {
    let config_dir = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|p| p.join(".config"))
                .expect("Cannot find HOME directory")
        });

    config_dir.join("ferristatus").join("config.json")
}

pub fn parse_test_config() -> Config {
    let args = Args {
        config_path: "examples/config.json".into(),
        ..Args::default()
    };

    Config::new(&args).expect("failed to get config")
}

pub fn de_vars_as_flat_hashmap<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    // First deserialize into a Vec of HashMaps
    let vec_maps = Vec::<HashMap<String, String>>::deserialize(deserializer)?;

    // Then flatten into a single HashMap
    let mut result = HashMap::new();
    for map in vec_maps {
        result.extend(map);
    }

    Ok(result)
}

pub fn safe_strfmt<K, T: DisplayStr>(template: &str, vars: &HashMap<K, T>) -> String
where
    K: Hash + Eq + FromStr + Display,
{
    match strfmt(template, vars) {
        Ok(formatted) => formatted,
        Err(_) => {
            // Return the original template
            template.to_string()
        }
    }
}

// /// Implement a new() struct for a Type using $type::default()
// #[macro_export]
// macro_rules! impl_new {
//     ($type:ident) => {
//         impl $type {
//             pub fn new() -> $type {
//                 $type::default()
//             }
//         }
//     };
// }
