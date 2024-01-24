use eframe::epaint::Color32;
use platform_dirs::AppDirs;
use serde::{Serialize, Deserialize};
use std::fs;
use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};

#[derive(Clone)]
pub struct ActualColors {
    pub bg_color: Color32,
    pub text_color: Color32,
    pub alert_bg_color: Color32,
}

impl Default for ActualColors {
    fn default() -> Self {
        Self {
            bg_color: Color32::from_hex("#000000FF").unwrap_or_default(),
            text_color: Color32::from_hex("#FFFFFFFF").unwrap_or_default(),
            alert_bg_color: Color32::from_hex("#00000080").unwrap_or_default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
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
    #[serde(default = "defaults::bg_color")]
    pub bg_color: String,
    #[serde(default = "defaults::text_color")]
    pub text_color: String,
    #[serde(default = "defaults::alert_bg_color")]
    pub alert_bg_color: String,
    #[serde(skip)]
    pub colors: ActualColors,
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
    default_!(bg_color, String);
    default_!(text_color, String);
    default_!(alert_bg_color, String);
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
            bg_color: "#000000FF".to_owned(),
            text_color: "#FFFFFFFF".to_owned(),
            alert_bg_color: "#00000080".to_owned(),
            colors: ActualColors::default(),
        }
    }
}

macro_rules! try_color {
    ($self:ident, $col:ident) => {
        let c = Color32::from_hex($self.$col.as_str());
        if let Ok(color) = c {
            $self.colors.$col = color;
        }
    };
}

impl EguiConfig {
    pub fn with_try_parse_colors(&mut self) -> Self {
        try_color!(self, bg_color);
        try_color!(self, text_color);
        try_color!(self, alert_bg_color);
        self.clone()
    }
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
                return Self::default().with_try_parse_colors()
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
        }.with_try_parse_colors()
    }
}
