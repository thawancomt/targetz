use directories::ProjectDirs;
use gpui_kit::{
    component::{Theme, ThemeRegistry},
    px, App, SharedString, Window,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// User-facing application settings persisted to `settings.json`.
///
/// Uses `#[serde(default)]` so that adding new fields later is
/// backward-compatible — old config files simply get the default value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    /// Theme name as shown in the ThemeRegistry (e.g. "Dark", "Catppuccin Mocha").
    pub theme_name: String,
    /// Border radius in pixels (0.0 = sharp corners, 6.0 = default rounded).
    pub radius: f32,
    /// Font family name (e.g. "Geist Mono", "Inter").
    pub font_family: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_name: "Dark".to_string(),
            radius: 6.0,
            font_family: "Geist Mono".to_string(),
        }
    }
}

/// Manages reading and writing application settings to disk.
///
/// Settings are persisted as a JSON file at the platform's standard config
/// directory (e.g. `~/.config/targetz/settings.json` on Linux).
pub struct SettingsManager;

impl SettingsManager {
    /// Returns the path to the settings file, creating parent directories
    /// if they don't exist.
    ///
    /// Uses `directories::ProjectDirs` for XDG compliance on Linux.
    /// Falls back to `~/.config/targetz/settings.json` if ProjectDirs fails.
    pub fn config_path() -> PathBuf {
        let config_dir = ProjectDirs::from("software", "whatever", "targetz")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".config").join("targetz")
            });

        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            eprintln!("Failed to create config directory {:?}: {}", config_dir, e);
        }

        config_dir.join("settings.json")
    }

    /// Reads and deserializes settings from disk.
    ///
    /// Returns `AppSettings::default()` on any error (missing file, invalid
    /// JSON, permission issues) so the app always boots successfully.
    pub fn read() -> AppSettings {
        let path = Self::config_path();

        match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
                eprintln!(
                    "Failed to parse settings from {:?}: {}. Using defaults.",
                    path, e
                );
                AppSettings::default()
            }),
            Err(_) => {
                // File doesn't exist yet — that's fine, use defaults.
                AppSettings::default()
            }
        }
    }

    /// Serializes and writes settings to disk as pretty-printed JSON.
    pub fn write(settings: &AppSettings) -> Result<(), String> {
        let path = Self::config_path();

        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write settings to {:?}: {}", path, e))
    }

    /// Reads settings from disk and applies theme + radius + font to the app.
    ///
    /// Designed for the pre-window startup phase — does not require a `Window`.
    /// After this call, `Theme::global(cx)` reflects the persisted preferences.
    ///
    /// **Ordering**: `apply_config` is called first (applies theme colors/mode),
    /// then radius and font_family are set (since `apply_config` would overwrite
    /// any values set before it).
    pub fn apply(cx: &mut App) {
        let settings = Self::read();
        Self::apply_settings(&settings, cx);
    }

    /// Reads settings from disk and applies them, then refreshes the window.
    ///
    /// Use this inside `open_window` callbacks where a `Window` is available.
    pub fn apply_with_window(window: &mut Window, cx: &mut App) {
        let settings = Self::read();
        Self::apply_settings(&settings, cx);
        window.refresh();
    }

    /// Core logic: look up the theme, apply config, then override radius/font.
    fn apply_settings(settings: &AppSettings, cx: &mut App) {
        let theme_name = SharedString::from(settings.theme_name.as_str());

        let config = {
            let registry = ThemeRegistry::global(cx);
            registry.themes().get(&theme_name).cloned()
        };

        if let Some(config) = config {
            // 1. Apply the theme config (colors, mode, highlight, etc.)
            Theme::global_mut(cx).apply_config(&config);

            // 2. Override radius AFTER apply_config (which would overwrite it)
            Theme::global_mut(cx).radius = px(settings.radius);

            // 3. Override font_family AFTER apply_config
            Theme::global_mut(cx).font_family = SharedString::from(settings.font_family.as_str());

            // 4. Push everything to the base layer (scrollbar, resize handles, etc.)
            Theme::sync_base(cx);
        } else {
            eprintln!(
                "Theme {:?} not found in registry. Keeping current theme.",
                settings.theme_name
            );
        }
    }

    /// Snapshots the current theme state and persists it to disk.
    ///
    /// Call this after any user-initiated theme/radius/font change in the UI.
    pub fn save_current(cx: &App) {
        let theme = Theme::global(cx);

        let settings = AppSettings {
            theme_name: theme.theme_name().to_string(),
            radius: f32::from(theme.radius),
            font_family: theme.font_family.to_string(),
        };

        if let Err(e) = Self::write(&settings) {
            eprintln!("Failed to persist settings: {}", e);
        }
    }

    /// Updates the theme's border radius, synchronizes the base layer,
    /// refreshes windows, and persists the setting.
    pub fn update_radius(radius: f32, cx: &mut App) {
        Theme::global_mut(cx).radius = px(radius);
        Theme::sync_base(cx);
        cx.refresh_windows();
        Self::save_current(cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn read_returns_default_when_file_missing() {
        // Point to a temp dir that doesn't have a settings file
        let settings = AppSettings::default();
        assert_eq!(settings.theme_name, "Dark");
        assert_eq!(settings.radius, 6.0);
        assert_eq!(settings.font_family, "Geist Mono");
    }

    #[test]
    fn write_then_read_round_trips() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = dir.path().join("settings.json");

        let original = AppSettings {
            theme_name: "Catppuccin Mocha".to_string(),
            radius: 12.0,
            font_family: "Inter".to_string(),
        };

        let json = serde_json::to_string_pretty(&original).unwrap();
        fs::write(&path, &json).unwrap();

        let read_back: AppSettings =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();

        assert_eq!(read_back.theme_name, "Catppuccin Mocha");
        assert_eq!(read_back.radius, 12.0);
        assert_eq!(read_back.font_family, "Inter");
    }

    #[test]
    fn read_returns_default_on_invalid_json() {
        let result: AppSettings =
            serde_json::from_str("{ not valid json }").unwrap_or_default();
        assert_eq!(result.theme_name, "Dark");
        assert_eq!(result.radius, 6.0);
    }

    #[test]
    fn read_handles_partial_json_with_serde_default() {
        // Only theme_name is present — radius and font_family should default
        let partial = r#"{ "theme_name": "Gruvbox Dark" }"#;
        let settings: AppSettings = serde_json::from_str(partial).unwrap();
        assert_eq!(settings.theme_name, "Gruvbox Dark");
        assert_eq!(settings.radius, 6.0);
        assert_eq!(settings.font_family, "Geist Mono");
    }
}
