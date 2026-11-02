//! Persisted desktop preferences. Kept free of UI dependencies so the state
//! machine and settings round-trips can be tested headlessly.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const APP_ID: &str = "app.figma2penpot.desktop";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeChoice {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// `None` means "next to each input file".
    pub output_dir: Option<PathBuf>,
    pub overwrite: bool,
    pub open_folder_after_conversion: bool,
    pub keep_token_bindings: bool,
    pub theme: ThemeChoice,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            output_dir: None,
            overwrite: false,
            open_folder_after_conversion: true,
            keep_token_bindings: true,
            theme: ThemeChoice::System,
        }
    }
}

impl Settings {
    /// Platform configuration path for this app id.
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(APP_ID)
            .join("settings.json")
    }

    /// Load settings, returning a warning instead of failing the app when the
    /// file is malformed.
    pub fn load(path: &Path) -> (Self, Option<String>) {
        match fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(settings) => (settings, None),
                Err(error) => (
                    Self::default(),
                    Some(format!("Could not read settings: {error}")),
                ),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (Self::default(), None),
            Err(error) => (
                Self::default(),
                Some(format!("Could not read settings: {error}")),
            ),
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, text)?;
        fs::rename(&temporary, path)
    }

    /// Resolve the output directory for one input. `None` keeps the result
    /// beside the source file.
    pub fn resolved_output_dir(&self, input: &Path) -> PathBuf {
        self.output_dir
            .clone()
            .or_else(|| input.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("settings.json");
        let settings = Settings {
            output_dir: Some(PathBuf::from("/tmp/out")),
            overwrite: true,
            open_folder_after_conversion: false,
            keep_token_bindings: true,
            theme: ThemeChoice::Dark,
        };
        settings.save(&path).unwrap();
        let (loaded, warning) = Settings::load(&path);
        assert_eq!(warning, None);
        assert_eq!(loaded, settings);
    }

    #[test]
    fn malformed_settings_fall_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, "{not json").unwrap();
        let (loaded, warning) = Settings::load(&path);
        assert_eq!(loaded, Settings::default());
        assert!(warning.is_some());
    }

    #[test]
    fn output_dir_defaults_to_input_parent() {
        let settings = Settings::default();
        assert_eq!(
            settings.resolved_output_dir(Path::new("/tmp/designs/a.fig")),
            PathBuf::from("/tmp/designs")
        );
    }
}
