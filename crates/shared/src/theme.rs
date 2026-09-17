use gpui_kit::{Hsla, rgb};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppColors {
    Clear,
    Caption,
    Background,
    Surface,
    Foreground,
    Inner,
    Border,
    Outline,
    Separator,
    AlternatingRow,
    IconTint,
    Text,
    Secondary,
    Group,
    File,
    Folder,
    Warning,
    Progress,
    Selection,
    RectSelection,
    Match,
    Hidden,
    Hover,
    Disabled,
    ContentHover,
    ContentSelection,
    ContentDisabledSelection,
    OutlineHover,
    OutlineSelection,
    OutlineDisabledSelection,
    MatchHover,
    MatchSelection,
    MatchDisabledSelection,
}

impl AppColors {
    /// Retorna as cores oficiais ajustadas ao estilo industrial/rust do File Pilot & TaskSlinger
    pub fn hsla(&self) -> Hsla {
        match self {
            // --- Fundo e Superfícies (Preto e Cinza Profundo Industrial) ---
            AppColors::Clear => rgb(0x111215).into(),
            AppColors::Caption => rgb(0x111215).into(),
            AppColors::Background => rgb(0x202122).into(),
            AppColors::Surface => rgb(0x1E2227).into(),
            AppColors::Inner => rgb(0x111215).into(),
            AppColors::AlternatingRow => rgb(0x1A1D22).into(),

            // --- Bordas e Divisores ---
            AppColors::Border => rgb(0x2D3139).into(),
            AppColors::Outline => rgb(0x3E4451).into(),
            AppColors::Separator => rgb(0x242830).into(),

            // --- Texto e Tipografia ---
            AppColors::Text => rgb(0xE6E6E6).into(),
            AppColors::Secondary => rgb(0x98A0A6).into(),
            AppColors::Foreground => rgb(0x282b2c).into(),

            // --- Elementos de UI (Pastas, Arquivos, Grupos) ---
            AppColors::IconTint => rgb(0xDE5D35).into(),
            AppColors::Group => rgb(0xB0B6BD).into(),
            AppColors::File => rgb(0xE6E6E6).into(),
            AppColors::Folder => rgb(0xDE5D35).into(),

            // --- Estados de Seleção e Foco ---
            AppColors::Selection => rgb(0x0078a4).into(),
            AppColors::RectSelection => rgb(0xDE5D35).into(),
            AppColors::Hover => rgb(0x173e4b).into(),
            AppColors::ContentHover => rgb(0xFFFFFF).into(),
            AppColors::ContentSelection => rgb(0xFFFFFF).into(),
            AppColors::ContentDisabledSelection => rgb(0x98A0A6).into(),

            // --- Feedbacks e Alertas ---
            AppColors::Warning => rgb(0xE06C75).into(),
            AppColors::Progress => rgb(0xDE5D35).into(),

            // --- Estados Avançados de Hover/Selection de Contornos ---
            AppColors::OutlineHover => rgb(0xDE5D35).into(),
            AppColors::OutlineSelection => rgb(0xDE5D35).into(),
            AppColors::OutlineDisabledSelection => rgb(0x4B5263).into(),

            // --- Filtros de Busca e Casamento de Padrões (Match) ---
            AppColors::Match => rgb(0xE5C07B).into(),
            AppColors::MatchHover => rgb(0xE5C07B).into(),
            AppColors::MatchSelection => rgb(0x16181C).into(),
            AppColors::MatchDisabledSelection => rgb(0x5C6370).into(),

            // --- Elementos Ocultos / Desativados ---
            AppColors::Hidden => rgb(0x4B5263).into(),
            AppColors::Disabled => rgb(0x3E4451).into(),
        }
    }
}
