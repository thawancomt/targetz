use gpui_kit::{
    base::{h_flex, v_flex, StyledExt},
    component::{
        button::{Button, ButtonVariants},
        input::{Input, InputEvent, InputState},
        scroll::ScrollableElement,
        ActiveTheme, Icon, IconName, Sizable, Theme, ThemeMode, ThemeRegistry,
    },
    div, px,
    prelude::FluentBuilder,
    AnyElement, App, AppContext, ClickEvent, Context, Entity, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window,
};

pub struct SettingsView {
    query_input: Entity<InputState>,
    query: String,
}

impl SettingsView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        if ThemeRegistry::global(cx).themes().len() <= 4 {
            crate::init_themes(cx);
        }
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let query_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Search themes (e.g. tokyo, catppuccin, gruvbox, light, dark)...")
        });

        cx.subscribe(&query_input, |this, _input, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                this.query = this.query_input.read(cx).value().to_string();
                cx.notify();
            }
        })
        .detach();

        Self {
            query_input,
            query: String::new(),
        }
    }

    fn filtered_theme_names(&self, cx: &Context<Self>) -> Vec<SharedString> {
        let registry = cx.global::<ThemeRegistry>();
        let sorted = registry.sorted_themes();
        let query = self.query.trim().to_lowercase();

        sorted
            .into_iter()
            .filter(|config| {
                query.is_empty()
                    || config.name.to_lowercase().contains(&query)
                    || config.mode.name().to_lowercase().contains(&query)
            })
            .map(|config| config.name.clone())
            .collect()
    }

    fn render_theme_row(
        &self,
        index: usize,
        theme_name: SharedString,
        is_current: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let label = theme_name.clone();

        h_flex()
            .id(("theme-row", index))
            .items_center()
            .gap_3()
            .px_3()
            .py_2()
            .rounded(theme.radius)
            .text_color(theme.popover_foreground)
            .bg(if is_current {
                theme.accent
            } else {
                theme.popover.opacity(0.)
            })
            .hover(|this| this.bg(theme.accent))
            .on_click(cx.listener({
                let name = theme_name.clone();
                move |this, _event: &ClickEvent, window, cx| {
                    this.apply_theme(&name, window, cx);
                }
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_7()
                    .rounded(theme.radius)
                    .bg(theme.primary.opacity(0.14))
                    .text_color(theme.primary)
                    .child(Icon::new(IconName::Palette).xsmall()),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .text_sm()
                    .text_ellipsis()
                    .overflow_hidden()
                    .child(label),
            )
            .when(is_current, |this| {
                this.child(
                    Icon::new(IconName::Check)
                        .xsmall()
                        .text_color(theme.primary),
                )
            })
            .into_any_element()
    }

    fn color_swatch(
        &self,
        name: &'static str,
        bg: gpui_kit::Hsla,
        fg: gpui_kit::Hsla,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        v_flex()
            .flex_1()
            .gap_1()
            .items_center()
            .child(
                div()
                    .w_full()
                    .h(px(34.))
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .bg(bg)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(div().text_xs().text_color(fg).child("Aa")),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(name),
            )
    }

    fn apply_theme(
        &mut self,
        theme_name: &SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let config = {
            let registry = cx.global::<ThemeRegistry>();
            registry.themes().get(theme_name).cloned()
        };

        if let Some(config) = config {
            Theme::global_mut(cx).font_family = "Geist Mono".into();
            Theme::global_mut(cx).apply_config(&config);
            Theme::sync_base(cx);
            window.refresh();
            cx.notify();
        }
    }

    fn toggle_theme_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let next_mode = if cx.theme().mode.is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };

        Theme::change(next_mode, Some(window), cx);
        Theme::global_mut(cx).font_family = "Geist Mono".into();
        Theme::sync_base(cx);
        window.refresh();
        cx.notify();
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let is_dark = theme.is_dark();
        let current_theme_name = theme.theme_name().clone();

        let filtered = self.filtered_theme_names(cx);
        let count = filtered.len();

        let row_elements: Vec<AnyElement> = filtered
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                let is_current = name == current_theme_name;
                self.render_theme_row(index, name, is_current, cx)
            })
            .collect();

        div()
            .id("settings-view-scroll")
            .size_full()
            .min_h_0()
            .min_w_0()
            .overflow_y_scrollbar()
            .bg(theme.background)
            .child(
                v_flex()
                    .w_full()
                    .items_center()
                    .py_12()
                    .px_6()
                    .child(
                        v_flex()
                            .w_full()
                            .max_w(px(680.))
                            .gap_6()
                            // Header
                            .child(
                                v_flex()
                                    .gap_1p5()
                                    .pb_2()
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_2p5()
                                            .child(
                                                div()
                                                    .p_2()
                                                    .rounded_lg()
                                                    .bg(theme.muted)
                                                    .child(Icon::new(IconName::Settings).size(px(22.))),
                                            )
                                            .child(
                                                div()
                                                    .text_2xl()
                                                    .font_bold()
                                                    .text_color(theme.foreground)
                                                    .child("Settings"),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.muted_foreground)
                                            .child("Configure application preferences and appearance"),
                                    ),
                            )
                            // Appearance Card
                            .child(
                                v_flex()
                                    .w_full()
                                    .p_5()
                                    .rounded_xl()
                                    .bg(theme.popover)
                                    .border_1()
                                    .border_color(theme.border)
                                    .gap_5()
                                    // Section header
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .justify_between()
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(Icon::new(IconName::Palette).size(px(16.)))
                                                    .child(
                                                        div()
                                                            .text_base()
                                                            .font_semibold()
                                                            .text_color(theme.foreground)
                                                            .child("Appearance"),
                                                    ),
                                            )
                                            .child(
                                                Button::new("theme-mode-switch")
                                                    .secondary()
                                                    .child(
                                                        h_flex()
                                                            .items_center()
                                                            .gap_1p5()
                                                            .child(
                                                                Icon::new(if is_dark {
                                                                    IconName::Sun
                                                                } else {
                                                                    IconName::Moon
                                                                })
                                                                .size(px(14.)),
                                                            )
                                                            .child(if is_dark {
                                                                "Switch to Light"
                                                            } else {
                                                                "Switch to Dark"
                                                            }),
                                                    )
                                                    .on_click(cx.listener(|this, _event, window, cx| {
                                                        this.toggle_theme_mode(window, cx);
                                                    })),
                                            ),
                                    )
                                    .child(div().h(px(1.)).w_full().bg(theme.border))
                                    // Search + count
                                    .child(
                                        v_flex()
                                            .w_full()
                                            .gap_3()
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .justify_between()
                                                    .child(
                                                        h_flex()
                                                            .items_center()
                                                            .gap_2()
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .font_medium()
                                                                    .text_color(theme.foreground)
                                                                    .child("Themes"),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .px_2()
                                                                    .py_0p5()
                                                                    .rounded_full()
                                                                    .bg(theme.muted)
                                                                    .text_color(theme.muted_foreground)
                                                                    .child(count.to_string()),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(theme.muted_foreground)
                                                            .child(format!("Active: {}", current_theme_name)),
                                                    ),
                                            )
                                            .child(Input::new(&self.query_input)),
                                    )
                                    // Theme rows container with fixed height and scrollbar
                                    .child(
                                        div()
                                            .id("theme-rows-container")
                                            .w_full()
                                            .h(px(320.))
                                            .p_1()
                                            .rounded_lg()
                                            .border_1()
                                            .border_color(theme.border)
                                            .bg(theme.background.opacity(0.6))
                                            .overflow_y_scrollbar()
                                            .child(
                                                v_flex()
                                                    .w_full()
                                                    .gap_1()
                                                    .when(row_elements.is_empty(), |this| {
                                                        this.child(
                                                            div()
                                                                .w_full()
                                                                .py_8()
                                                                .flex()
                                                                .items_center()
                                                                .justify_center()
                                                                .text_sm()
                                                                .text_color(theme.muted_foreground)
                                                                .child("No matching themes found"),
                                                        )
                                                    })
                                                    .children(row_elements),
                                            ),
                                    )
                                    // Palette preview
                                    .child(
                                        v_flex()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_semibold()
                                                    .text_color(theme.muted_foreground)
                                                    .child("ACTIVE PALETTE PREVIEW"),
                                            )
                                            .child(
                                                h_flex()
                                                    .w_full()
                                                    .gap_2()
                                                    .child(self.color_swatch("Primary", theme.primary, theme.primary_foreground, cx))
                                                    .child(self.color_swatch("Background", theme.background, theme.foreground, cx))
                                                    .child(self.color_swatch("Surface", theme.popover, theme.foreground, cx))
                                                    .child(self.color_swatch("Accent", theme.accent, theme.accent_foreground, cx))
                                                    .child(self.color_swatch("Muted", theme.muted, theme.muted_foreground, cx)),
                                            ),
                                    ),
                            ),
                    ),
            )
    }
}
