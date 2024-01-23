use platform_dirs::AppDirs;
use serde::{Serialize, Deserialize};
use std::fs;
use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};

#[derive(Serialize, Deserialize)]
pub struct EguiConfig {
    #[serde(default = "defaults::window_fits_content")]
    pub window_fits_content: bool,
    #[serde(default = "defaults::min_window_size")]
    pub min_window_size: u32,
    #[serde(default = "defaults::max_window_size")]
    pub max_window_size: u32,
    #[serde(default = "defaults::always_on_top")]
    pub always_on_top: bool,
    #[serde(default = "defaults::window_decorated")]
    pub window_decorated: bool,
    #[serde(default = "defaults::vars_alert_time")]
    pub vars_alert_time: f32,
    #[serde(default = "defaults::copy_eq_alert_time")]
    pub copy_eq_alert_time: f32,
    #[serde(default = "defaults::base_change_alert_time")]
    pub base_change_alert_time: f32,
}

macro_rules! default_ {
    ($name:ident, $type:ident) => {
        pub fn $name() -> $type {
            EguiConfig::default().$name
        }
    };
}

mod defaults {
    use super::EguiConfig;
    default_!(window_fits_content, bool);
    default_!(min_window_size, u32);
    default_!(max_window_size, u32);
    default_!(always_on_top, bool);
    default_!(window_decorated, bool);
    default_!(vars_alert_time, f32);
    default_!(copy_eq_alert_time, f32);
    default_!(base_change_alert_time, f32);
}

impl Default for EguiConfig {
    fn default() -> Self {
        Self {
            window_fits_content: false,
            min_window_size: 300,
            max_window_size: 1920,
            always_on_top: false,
            window_decorated: false,
            vars_alert_time: 2.,
            copy_eq_alert_time: 1.5,
            base_change_alert_time: 1.,
        }
    }
}

impl EguiConfig {
    pub fn load() -> Self {
        let dirs = AppDirs::new(Some("minicalc"), false).unwrap();
        let config_dir = dirs.config_dir;
        let _ = fs::create_dir_all(&config_dir);
        let egui_config_path = config_dir.join("egui.cfg");
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
