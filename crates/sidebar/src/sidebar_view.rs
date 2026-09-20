use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, IntoElement, ParentElement, Render, Styled,
    Window,
    base::{StyledExt, h_flex, v_flex},
    component::{
        ActiveTheme, IconName, Theme, ThemeMode,
        button::{Button, ButtonVariants},
        scroll::ScrollableElement,
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
                    .child(self.sidebar_item(AppTab::CreateProject, cx)),
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
