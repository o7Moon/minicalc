use platform_dirs::AppDirs;
use serde::{Serialize, Deserialize};
use ron::ser::{to_string_pretty, PrettyConfig};
use ron::de::from_reader;
use std::fs;
use crate::math::base::NumberBase;

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "defaults::max_fractional_places")]
    pub max_fractional_places: u32,
    #[serde(default = "defaults::base")]
    pub base: NumberBase,
}

macro_rules! default_ {
    ($name:ident, $type:ident) => {
        pub fn $name() -> $type {
            Config::default().$name
        }
    };
}

mod defaults {
    use super::Config;
    use super::NumberBase;
    default_!(max_fractional_places, u32);
    default_!(base, NumberBase);
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_fractional_places: 128,
            base: NumberBase::Decimal,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let dirs = AppDirs::new(Some("minicalc"), false).unwrap();
        let config_dir = dirs.config_dir;
        let _ = fs::create_dir_all(&config_dir);
        let egui_config_path = config_dir.join("minicalc.cfg");
        let file = fs::File::open(&egui_config_path);
        let file = match file {
            Ok(file) => {file},
            Err(_) => {
                _ = fs::write(egui_config_path, to_string_pretty(&Self::default(), PrettyConfig::default()).unwrap());
                return Self::default()
            }
        };
        let conf = from_reader::<fs::File, Self>(file);
        match conf {
            Ok(conf) => {
                // write back default values of any fields not present
                _ = fs::write(egui_config_path, to_string_pretty(&conf, PrettyConfig::default()).unwrap());
                conf
            },
            Err(_) => Self::default(),
        } 
    }
}
