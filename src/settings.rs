use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub folders: Vec<PathBuf>,
    pub favourites: BTreeSet<PathBuf>,
    pub volume: f32,
    pub shuffle: bool,
    pub repeat: crate::queue::Repeat,
    /// What order the shelves are listed in. Remembered, unlike the film
    /// player's and the photo viewer's, because those two are opened on a
    /// folder somebody chose and this one is opened on a library that is
    /// always the same: an order set here is a statement about how you like
    /// your own music listed, not about the folder you happen to be in.
    pub order: crate::library::Order,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            folders: vec![music_folder()],
            favourites: BTreeSet::new(),
            volume: 0.75,
            shuffle: false,
            repeat: crate::queue::Repeat::Off,
            order: crate::library::Order::default(),
        }
    }
}

pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
}

pub fn music_folder() -> PathBuf {
    // user-dirs.dirs is data, never shell code. Expand only its documented HOME form.
    if let Ok(dirs) = std::fs::read_to_string(config_dir().join("user-dirs.dirs")) {
        for line in dirs.lines() {
            if let Some(value) = line.trim().strip_prefix("XDG_MUSIC_DIR=") {
                let value = value.trim().trim_matches('"');
                if value == "$HOME" {
                    return home();
                }
                if let Some(rest) = value.strip_prefix("$HOME/") {
                    return home().join(rest);
                }
                let path = PathBuf::from(value);
                if path.is_absolute() {
                    return path;
                }
            }
        }
    }
    home().join("Music")
}

impl Settings {
    pub fn load() -> Self {
        let path = config_dir().join("songonsole/settings.json");
        let mut settings: Self = std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        settings.volume = if settings.volume.is_finite() {
            settings.volume.clamp(0.0, 1.0)
        } else {
            0.75
        };
        settings
    }

    pub fn save(&self) -> Result<(), String> {
        let directory = config_dir().join("songonsole");
        std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        let temporary = directory.join(format!("settings.{}.tmp", std::process::id()));
        std::fs::write(
            &temporary,
            serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        std::fs::rename(temporary, directory.join("settings.json")).map_err(|e| e.to_string())
    }
}
