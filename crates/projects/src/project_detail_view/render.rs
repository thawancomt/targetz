use gpui_kit::{
    ParentElement, Render, Styled,
    component::{ActiveTheme, button::Button, scroll::ScrollableElement},
    div,
};
use shared::project::Project;

use crate::project_detail_view::state::ProjectDetailView;

impl Render for ProjectDetailView {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let Project {
            name,
            current_version,
            created_at: _,
            status,
            description: _,
            start_date: _,
            site_url: _,
            codename,
            target_deadline: _,
            updated_at: _,
            budget: _,
            ..
        } = self.project.clone();

        let theme = cx.theme();

        div()
            .size_full()
            .h_full()
            .overflow_y_scrollbar()
            .child(
                div()
                    .p_2()
                    .child(div().bg(theme.primary).w_48().h_48())
                    .child(div().child(name).child(format!(
                        "{} • {}",
                        current_version,
                        codename.unwrap_or_default()
                    )))
                    .child(
                        div().child(format!("Status : {}", status.as_str())).child(
                            div()
                                .child(Button::new("Finish").label("Finish"))
                                .child(Button::new("Prospecting").label("Prospecting"))
                                .child(Button::new("Canceled").label("Canceled")),
                        ),
                    ),
            )
            .child(
                div()
                    .child(
                        div()
                            .child("Project history")
                            .text_lg()
                            .text_color(theme.primary),
                    )
                    .child(
                        div()
                            .border_1()
                            .border_color(theme.border)
                            .flex_1()
                            .child("Project history"),
                    ),
            )
    }
}
