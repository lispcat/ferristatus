use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

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
