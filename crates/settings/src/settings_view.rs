use gpui_kit::{
    App, AppContext, Context, Entity, ParentElement, Render, Styled, Window,
    component::{
        ActiveTheme,
        input::{Input, InputState},
    },
    div,
};

pub struct SettingsView {
    start_with_windows: bool,
    start_with_windows_input: Entity<InputState>,
}

impl Render for SettingsView {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        div()
            .flex_1()
            .flex_col()
            .child(
                div()
                    .w_full()
                    .bg(cx.theme().border)
                    .child("Targetz Settings"),
            )
            .child(Input::new(&self.start_with_windows_input))
            .text_color(cx.theme().primary)
    }
}

impl SettingsView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let start_with_windows_input = cx.new(|cx| InputState::new(window, cx));

        Self {
            start_with_windows: false,
            start_with_windows_input: start_with_windows_input,
        }
    }
}
