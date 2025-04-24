use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

use crate::utils::safe_strfmt;

pub trait VarPreprocessing {
    fn get_levels(&self) -> &Vec<(i32, String)>;

    fn safe_strfmt_levels(&self, vars: &HashMap<String, String>) -> Vec<(i32, String)> {
        let mut levels: Vec<(i32, String)> = self
            .get_levels()
            .clone()
            .into_iter()
            .map(|(k, v)| (k, safe_strfmt(&v, vars)))
            .collect();
        levels.sort_by_key(|(k, _)| *k);
        levels
    }
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
