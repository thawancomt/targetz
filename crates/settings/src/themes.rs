use gpui_kit::{
    component::ThemeRegistry,
    App,
};

pub const THEME_CONTENTS: &[&str] = &[
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
    include_str!("../../../themes/app-theme.json"),
];

pub fn init_themes(cx: &mut App) {
    let registry = ThemeRegistry::global_mut(cx);
    for content in THEME_CONTENTS {
        if let Err(err) = registry.load_themes_from_str(content) {
            eprintln!("Failed to load embedded theme: {}", err);
        }
    }
}
