use crate::error::{GimError, Result};
use directories::ProjectDirs;
use ratatui::style::Color;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub print: PrintConfig,
    pub tui: TuiConfig,
    pub theme: ThemeConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct GeneralConfig {
    pub refresh_ms: u64,
    pub default_modules: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct PrintConfig {
    pub output: String,
    pub show_units: bool,
    pub watch: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct TuiConfig {
    pub refresh_ms: Option<u64>,
    pub borders: BorderStyle,
    pub show_help: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum BorderStyle {
    None,
    Plain,
    Rounded,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ThemeConfig {
    pub cpu: ModuleTheme,
    pub memory: ModuleTheme,
    pub disk: ModuleTheme,
    pub network: ModuleTheme,
    pub process: ModuleTheme,
    pub system: ModuleTheme,
    pub chrome: ChromeTheme,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ModuleTheme {
    pub label: String,
    pub fg: String,
    pub accent: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ChromeTheme {
    pub border: String,
    pub title: String,
    pub header: String,
    pub error: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            print: PrintConfig::default(),
            tui: TuiConfig::default(),
            theme: ThemeConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            refresh_ms: 1000,
            default_modules: vec![
                "system".into(),
                "cpu".into(),
                "memory".into(),
                "disk".into(),
                "network".into(),
                "process".into(),
            ],
        }
    }
}

impl Default for PrintConfig {
    fn default() -> Self {
        Self {
            output: "table".into(),
            show_units: true,
            watch: false,
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            refresh_ms: None,
            borders: BorderStyle::Rounded,
            show_help: true,
        }
    }
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle::Rounded
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            cpu: ModuleTheme {
                label: "CPU".into(),
                fg: "cyan".into(),
                accent: "light_cyan".into(),
            },
            memory: ModuleTheme {
                label: "Memory".into(),
                fg: "green".into(),
                accent: "light_green".into(),
            },
            disk: ModuleTheme {
                label: "Disk".into(),
                fg: "yellow".into(),
                accent: "light_yellow".into(),
            },
            network: ModuleTheme {
                label: "Network".into(),
                fg: "magenta".into(),
                accent: "light_magenta".into(),
            },
            process: ModuleTheme {
                label: "Processes".into(),
                fg: "red".into(),
                accent: "light_red".into(),
            },
            system: ModuleTheme {
                label: "System".into(),
                fg: "blue".into(),
                accent: "light_blue".into(),
            },
            chrome: ChromeTheme::default(),
        }
    }
}

impl Default for ModuleTheme {
    fn default() -> Self {
        Self {
            label: String::new(),
            fg: "white".into(),
            accent: "gray".into(),
        }
    }
}

impl Default for ChromeTheme {
    fn default() -> Self {
        Self {
            border: "gray".into(),
            title: "white".into(),
            header: "white".into(),
            error: "red".into(),
        }
    }
}

impl Config {
    pub fn tui_refresh_ms(&self) -> u64 {
        self.tui.refresh_ms.unwrap_or(self.general.refresh_ms)
    }
}

pub fn default_config_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "gim").map(|dirs| dirs.config_dir().join("gim_config.yaml"))
}

pub fn load_config(path: Option<&Path>) -> Result<Config> {
    let config_path = match path {
        Some(p) => {
            if p.exists() {
                Some(p.to_path_buf())
            } else {
                return Err(GimError::ConfigLoad {
                    path: p.display().to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "config file not found",
                    )),
                });
            }
        }
        None => default_config_path().filter(|p| p.exists()),
    };

    match config_path {
        Some(path) => {
            let contents = std::fs::read_to_string(&path).map_err(|e| GimError::ConfigLoad {
                path: path.display().to_string(),
                source: Box::new(e),
            })?;
            let config: Config = serde_yaml::from_str(&contents)?;
            Ok(config)
        }
        None => Ok(Config::default()),
    }
}

pub fn parse_color(color_str: &str) -> Color {
    match color_str.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "dark_gray" | "dark_grey" => Color::DarkGray,
        "light_red" => Color::LightRed,
        "light_green" => Color::LightGreen,
        "light_yellow" => Color::LightYellow,
        "light_blue" => Color::LightBlue,
        "light_magenta" => Color::LightMagenta,
        "light_cyan" => Color::LightCyan,
        "white" => Color::White,
        hex if hex.starts_with('#') && hex.len() == 7 => {
            let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(255);
            let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(255);
            Color::Rgb(r, g, b)
        }
        _ => Color::White,
    }
}
