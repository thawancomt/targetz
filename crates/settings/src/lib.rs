pub mod settings_manager;
pub mod settings_view;
pub mod themes;

pub use settings_manager::{AppSettings, SettingsManager};
pub use themes::{init_themes, THEME_CONTENTS};

#[cfg(test)]
mod tests {
    use gpui_kit::component::ThemeRegistry;

    #[test]
    fn test_load_all_themes() {
        let theme_contents = [
            include_str!("../../../assets/themes/adventure.json"),
            include_str!("../../../assets/themes/alduin.json"),
            include_str!("../../../assets/themes/asciinema.json"),
            include_str!("../../../assets/themes/aurora.json"),
            include_str!("../../../assets/themes/ayu.json"),
            include_str!("../../../assets/themes/catppuccin.json"),
            include_str!("../../../assets/themes/everforest.json"),
            include_str!("../../../assets/themes/fahrenheit.json"),
            include_str!("../../../assets/themes/flexoki.json"),
            include_str!("../../../assets/themes/gruvbox.json"),
            include_str!("../../../assets/themes/harper.json"),
            include_str!("../../../assets/themes/hybrid.json"),
            include_str!("../../../assets/themes/jellybeans.json"),
            include_str!("../../../assets/themes/kibble.json"),
            include_str!("../../../assets/themes/macos-classic.json"),
            include_str!("../../../assets/themes/mellifluous.json"),
            include_str!("../../../assets/themes/molokai.json"),
            include_str!("../../../assets/themes/solarized.json"),
            include_str!("../../../assets/themes/spaceduck.json"),
            include_str!("../../../assets/themes/tokyonight.json"),
            include_str!("../../../assets/themes/twilight.json"),
        ];

        let mut registry = ThemeRegistry::default();
        for (i, content) in theme_contents.iter().enumerate() {
            let res = registry.load_themes_from_str(content);
            assert!(res.is_ok(), "Failed to load theme #{}: {:?}", i, res.err());
        }
        println!("Loaded themes count: {}", registry.themes().len());
        for name in registry.themes().keys() {
            println!("Theme: {}", name);
        }
    }
}
