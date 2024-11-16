#![allow(dead_code)]
use std::path::PathBuf;
use std::{env, fs, io};

use serde::{Deserialize, Serialize};

use crate::theme::{default_theme, Theme};

fn get_user_home_dir() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        // On Windows, check the `USERPROFILE` or `HOMEDRIVE` + `HOMEPATH`
        env::var("USERPROFILE")
            .or_else(|_| {
                let homedrive = env::var("HOMEDRIVE");
                let homepath = env::var("HOMEPATH");
                match (homedrive, homepath) {
                    (Ok(drive), Ok(path)) => Ok(format!("{}{}", drive, path)),
                    _ => Err(env::VarError::NotPresent),
                }
            })
            .ok()
            .map(PathBuf::from)
    } else {
        // On Unix-like systems (Linux, macOS), check the `HOME` environment variable
        env::var("HOME").ok().map(PathBuf::from)
    }
}

pub fn read_editor_settings() -> io::Result<Settings> {
    let mut home_dir = match get_user_home_dir() {
        Some(dir) => dir,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "User directory not found.",
            ))
        }
    };

    home_dir.push(".revo");
    if !home_dir.exists() {
        fs::create_dir_all(&home_dir)?;
    }

    let mut settings_file_path = home_dir.clone();
    settings_file_path.push("settings.json");

    let mut themes_path = home_dir;
    themes_path.push("themes");

    let settings_schema = if !settings_file_path.exists() {
        // Generate default settings
        let default_settings = default_settings_schema();
        
        // Save settings
        let default_settings_str = serde_json::to_string_pretty(&default_settings).unwrap();
        fs::write(&settings_file_path, default_settings_str)?;

        default_settings
    } else {
        let settings_string = fs::read_to_string(settings_file_path)?;
        match serde_json::from_str(&settings_string) {
            Ok(settings) => settings,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput, e)),
        }
    };

    let theme = if !themes_path.exists() {
        // Create themes folder
        fs::create_dir_all(&themes_path)?;
        // Generate default Theme
        let default_theme = default_theme();
        // Save to folder
        let mut default_theme_file_path = themes_path;
        default_theme_file_path.push("default.json");

        let default_theme_str = serde_json::to_string_pretty(&default_theme).unwrap();
        fs::write(default_theme_file_path, default_theme_str)?;

        default_theme
    } else {
        // Get path to selected theme
        let mut theme_path = themes_path;
        theme_path.push(format!("{}.json", settings_schema.active_theme));
        if !theme_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Theme file not found."));
        }
        
        // Read them to themes list
        let theme_str = fs::read_to_string(theme_path)?;
        match serde_json::from_str(&theme_str) {
            Ok(theme) => theme,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput, e)),
        }
    };
    
    Ok(Settings {
        theme,
    })
}

#[derive(Serialize, Deserialize, Debug)]
struct SettingsSchema {
    /// Name of the active theme
    pub active_theme: String,

    // Add settings in here
}

fn default_settings_schema() -> SettingsSchema {
    SettingsSchema {
        active_theme: "default".to_string(),
    }
}

#[derive(Debug)]
pub struct Settings {
    pub theme: Theme,
}
