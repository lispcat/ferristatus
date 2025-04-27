mod alsa;
mod backlight;
mod battery;
mod time;
mod utils;

pub use component_traits::*;

pub mod component_traits {

    use crate::utils::safe_strfmt;
    use serde::{Deserialize, Deserializer, de};
    use std::{
        collections::HashMap,
        error::Error,
        fmt::{Debug, Display},
    };

    pub trait Component: Debug + Display {
        fn name(&self) -> String;
        fn update(&mut self) -> anyhow::Result<()>;
        fn get_format_string(&self) -> String;
        fn eval_strfmt(&self, format_str: &str) -> anyhow::Result<String>;
        fn new_from_settings<T>(settings: T) -> Self
        where
            T: ComponentSettings + Into<Self>,
            Self: Sized,
        {
            settings.into()
        }
    }

    pub trait ComponentSettings: Debug + for<'a> Deserialize<'a> {}

    pub trait ComponentFormatSettings: Debug {
        /// Custom deserializer that does strfmt preprocessing with vars
        fn de_strfmt<'de, D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
            Self: Sized + Deserialize<'de>,
        {
            // Deserialize raw settings
            let format = Self::deserialize(deserializer)?;

            // copy from vars field
            let vars = format.get_vars().clone();

            // Apply formatting transformations
            let formatted = Self::de_strfmt_formatting(&format, vars)
                .map_err(de::Error::custom)?;

            Ok(formatted)
        }

        fn get_vars(&self) -> &HashMap<String, String>;

        fn get_levels(&self) -> Option<&Vec<(i32, String)>>;

        fn de_strfmt_formatting<'de>(
            orig: &Self,
            vars: HashMap<String, String>,
        ) -> Result<Self, String>
        where
            Self: Sized;

        fn safe_strfmt_levels(
            &self,
            vars: &HashMap<String, String>,
        ) -> anyhow::Result<Vec<(i32, String)>, Box<dyn Error>> {
            let mut levels: Vec<(i32, String)> = self
                .get_levels()
                .ok_or("no levels found".to_string())?
                .clone()
                .into_iter()
                .map(|(k, v)| (k, safe_strfmt(&v, vars)))
                .collect();
            levels.sort_by_key(|(k, _)| *k);
            Ok(levels)
        }
    }

    pub trait ComponentState: Debug {}
}

pub use component_list::*;

pub mod component_list {

    use std::{collections::HashMap, error::Error};

    use serde::{Deserialize, Deserializer};
    use serde_yml::Value;
    use smart_default::SmartDefault;

    use super::{
        alsa::{Alsa, AlsaSettings},
        backlight::{Backlight, BacklightSettings},
        battery::{Battery, BatterySettings},
        time::{Time, TimeSettings},
        *,
    };

    #[derive(SmartDefault, Debug)]
    pub struct ComponentList {
        #[default(Vec::new())]
        pub list: Vec<Box<dyn Component>>,
    }

    fn component_create(key: &String, value: &Value) -> Result<Box<dyn Component>, Box<dyn Error>> {
        match key.to_lowercase().as_str() {
            "alsa" => {
                let settings: AlsaSettings = serde_yml::from_value(value.clone())
                    .map_err(|_| format!("failed to parse {} config", key))?;
                Ok(Box::new(Alsa::new_from_settings(settings)))
            }
            "backlight" => {
                let settings: BacklightSettings = serde_yml::from_value(value.clone())
                    .map_err(|_| format!("failed to parse {} config", key))?;
                Ok(Box::new(Backlight::new_from_settings(settings)))
            }
            "battery" => {
                let settings: BatterySettings = serde_yml::from_value(value.clone())
                    .map_err(|_| format!("failed to parse {} config", key))?;
                Ok(Box::new(Battery::new_from_settings(settings)))
            }
            "time" => {
                let settings: TimeSettings = serde_yml::from_value(value.clone())
                    .map_err(|_| format!("failed to parse {} config", key))?;
                Ok(Box::new(Time::new_from_settings(settings)))
            }
            _ => Err(format!("can't parse unknown component {}", key).into()),
        }
    }

    impl<'de> Deserialize<'de> for ComponentList {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Deserialize into a vec of length-one hashmaps first
            let components_vec: Vec<HashMap<String, Value>> = Vec::deserialize(deserializer)?;

            let components_flattened: Vec<(String, Value)> = components_vec
                .into_iter()
                .flat_map(|map| map.into_iter())
                .collect();

            // Parse each component
            let component_list = components_flattened
                .iter()
                .map(|(component_name, settings)| {
                    component_create(component_name, settings).map_err(|e| {
                        serde::de::Error::custom(format!(
                            "could not parse component {}: {}",
                            component_name, e
                        ))
                    })
                })
                .collect::<Result<Vec<Box<dyn Component>>, D::Error>>()?;

            Ok(ComponentList {
                list: component_list,
            })
        }
    }
}
