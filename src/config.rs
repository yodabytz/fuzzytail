use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub themes: ThemeConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralConfig {
    pub theme: String,
    pub buffer_size: Option<usize>,
    pub follow_retry_interval: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ThemeConfig {
    pub builtin_path: PathBuf,
    pub user_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                theme: "catppuccin".to_string(),
                buffer_size: Some(8192),
                follow_retry_interval: Some(1000),
            },
            themes: ThemeConfig {
                builtin_path: PathBuf::from("/etc/fuzzytail/themes"),
                user_path: dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("~/.config"))
                    .join("fuzzytail/themes"),
            },
        }
    }
}

impl Config {
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        let config_file = if let Some(path) = config_path {
            path.to_path_buf()
        } else {
            Self::default_config_path()?
        };

        if config_file.exists() {
            let contents = fs::read_to_string(&config_file)
                .with_context(|| format!("Failed to read config file: {:?}", config_file))?;
            
            let config: Config = toml::from_str(&contents)
                .with_context(|| format!("Failed to parse config file: {:?}", config_file))?;
            
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            Self::ensure_config_dir(&config_file)?;
            Self::save_default_config(&config_file, &config)?;
            Ok(config)
        }
    }

    fn default_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to find config directory")?
            .join("fuzzytail");
        
        Ok(config_dir.join("config.toml"))
    }

    fn ensure_config_dir(config_file: &Path) -> Result<()> {
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }
        Ok(())
    }

    fn save_default_config(config_file: &Path, config: &Config) -> Result<()> {
        let contents = toml::to_string_pretty(config)
            .context("Failed to serialize default config")?;
        
        fs::write(config_file, contents)
            .with_context(|| format!("Failed to write default config: {:?}", config_file))?;
        
        println!("Created default config at: {:?}", config_file);
        Ok(())
    }

    pub fn get_theme_path(&self, theme_name: &str) -> Option<PathBuf> {
        let theme_file = format!("ft.conf.{}", theme_name);
        
        // Check user path first
        let user_theme = self.themes.user_path.join(&theme_file);
        if user_theme.exists() {
            return Some(user_theme);
        }
        
        // Check builtin path
        let builtin_theme = self.themes.builtin_path.join(&theme_file);
        if builtin_theme.exists() {
            return Some(builtin_theme);
        }
        
        None
    }
}