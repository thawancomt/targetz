use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, IntoElement, ParentElement, Render, Styled,
    Window,
    base::{StyledExt, h_flex},
    component::{
        ActiveTheme, Icon, IconName, Theme, ThemeMode,
        button::{Button, ButtonVariants},
        switch::Switch,
    },
    div,
    prelude::FluentBuilder,
    px,
};
use shared::AppTab;

pub enum SidebarEvent {
    TabClick(AppTab),
}

pub struct SidebarView {
    pub active_tab: AppTab,
}

impl EventEmitter<SidebarEvent> for SidebarView {}

impl Render for SidebarView {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        h_flex()
            .h_full()
            .w(px(300.))
            .bg(cx.theme().border)
            .p_2()
            .flex_col()
            .child(
                div()
                    .v_flex()
                    .flex_1()
                    .w_full()
                    .gap_1()
                    .child(self.sidebar_item(AppTab::Home, cx))
                    .child(self.sidebar_item(AppTab::Targetz, cx))
                    .child(self.sidebar_item(AppTab::CreateCustomer, cx))
                    .child(self.sidebar_item(AppTab::Settings, cx))
                    .child(self.sidebar_item(AppTab::Projects, cx)),
            )
            .child(
                div()
                    .pt_2()
                    .w_full()
                    .child(self.theme_toggle_item(cx)),
            )
    }
}

impl SidebarView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            active_tab: AppTab::Home,
        }
    }

    pub fn toggle_tab(&mut self, cx: &mut Context<Self>, tab: AppTab) {
        self.active_tab = tab.clone();
        cx.emit(SidebarEvent::TabClick(tab));
        cx.notify();
    }

    pub fn toggle_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let is_dark = cx.theme().mode.is_dark();
        let next_mode = if is_dark {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };

        Theme::change(next_mode, Some(window), cx);
        Theme::global_mut(cx).font_family = "Geist Mono".into();
        cx.notify();
    }

    fn theme_toggle_item(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_dark = cx.theme().mode.is_dark();
        let (label, icon) = if is_dark {
            ("Dark mode", IconName::Moon)
        } else {
            ("Light mode", IconName::Sun)
        };

        Button::new("theme-mode-toggle")
            .w_full()
            .secondary()
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new(icon))
                            .child(label),
                    )
                    .child(Switch::new("theme-switch").checked(is_dark)),
            )
            .on_click(cx.listener(|this, _event, window, cx| {
                this.toggle_theme(window, cx);
            }))
    }

    fn sidebar_item(&mut self, tab: AppTab, cx: &mut Context<Self>) -> impl IntoElement {
        Button::new(format!("{}", tab.as_str()))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .justify_between()
                    .child(format!("{}", tab.as_str()))
                    .child(IconName::ArrowRight),
            )
            .w_full()
            .secondary()
            .when(self.active_tab == tab, |e| e.primary())
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.toggle_tab(cx, tab.clone());
            }))
    }
}
