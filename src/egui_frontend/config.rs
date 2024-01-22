use platform_dirs::AppDirs;
use serde::{Serialize, Deserialize};
use std::fs;
use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};

#[derive(Serialize, Deserialize)]
pub struct EguiConfig {
    #[serde(default)]
    pub window_fits_content: bool,
    #[serde(default)]
    pub min_window_size: u32,
    #[serde(default)]
    pub max_window_size: u32,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default)]
    pub window_decorated: bool,
}

impl Default for EguiConfig {
    fn default() -> Self {
        Self {
            window_fits_content: false,
            min_window_size: 300,
            max_window_size: 1920,
            always_on_top: false,
            window_decorated: false,
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
            Ok(conf) => conf,
            Err(_) => Self::default(),
        }
    }
}
