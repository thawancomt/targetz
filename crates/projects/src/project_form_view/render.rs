use gpui_kit::{
    App, Entity, ParentElement, Render, Styled,
    base::input::InputState,
    component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
        date_picker::{DatePicker, DatePickerState},
        form::Field,
        input::Input,
        scroll::ScrollableElement,
        select::Select,
    },
    div, px,
};
use shared::form_utils::{app_date_field, app_field};

use crate::project_form_view::state::CreateProjectView;

fn text(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).text().to_string()
}

impl Render for CreateProjectView {
    fn render(
        &mut self,
        window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let theme = cx.theme();
        let project_name = text(&self.name, cx);

        let wide = window.viewport_size().width >= px(768. + 300.);

        div()
            .p_2()
            .flex()
            .flex_col()
            .min_h_0()
            .overflow_y_scrollbar()
            .child(
                div()
                    .flex()
                    .text_lg()
                    .child(format!("[{project_name}]"))
                    .text_color(theme.accent_foreground),
            )
            .child(
                div()
                    .grid()
                    .grid_cols(if wide { 2 } else { 1 })
                    .gap_2()
                    .child(app_field("Name", &self.name).rounded(px(0.)))
                    .child(app_field("Project Codename", &self.codename))
                    .child(app_field("Description", &self.description))
                    .child(app_field("Site url", &self.site_url))
                    .child(app_field("Version", &self.version))
                    .child(app_field("Budget", &self.budget))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(app_date_field("Project due", &self.target_deadline))
                            .child(app_date_field("Est. Start date", &self.start_date)),
                    )
                    .child(
                        Field::new()
                            .child(Select::new(&self.status))
                            .label("Status"),
                    )
                    .child(
                        Field::new()
                            .child(Select::new(&self.owner))
                            .label("Project Owner"),
                    )
                    .child(
                        Button::new("Creat button")
                            .primary()
                            .col_span_full()
                            .label("Create")
                            .on_click(cx.listener(|view, _, _, cx| view.create_project(cx)))
                            .mt_2(),
                    ),
            )
    }
}
