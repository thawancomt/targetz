use gpui_kit::{
    IntoElement, ParentElement, Render, Styled,
    base::Disableable,
    component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
        list::List,
        scroll::ScrollableElement,
    },
    div,
    prelude::FluentBuilder,
    px,
};
use shared::project::{Project, ProjectStatus};

use crate::project_detail_view::state::ProjectDetailView;

impl Render for ProjectDetailView {
    fn render(
        &mut self,
        window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let Some(ref project) = self.project else {
            return div().into_any_element();
        };

        let Project {
            name,
            current_version,
            created_at: _,
            status,
            description,
            start_date: _,
            site_url: _,
            codename,
            target_deadline: _,
            updated_at: _,
            budget: _,
            ..
        } = project.clone();

        let theme = cx.theme();

        let button = |project_status: ProjectStatus| -> Button {
            Button::new(project_status.as_str())
                .label(project_status.as_str())
                .when(status == project_status, |b| b.primary())
                .on_click(cx.listener(move |this, _emitter, _ev, cx| {
                    this.toggle_status(project_status.clone(), cx);
                    cx.notify();
                }))
        };

        div()
            .size_full()
            .h_full()
            .overflow_y_scrollbar()
            .child(
                div()
                    .flex()
                    .p_2()
                    .child(div().bg(theme.primary).w_48().h_48())
                    .child(
                        div().p_2().flex().flex_1().bg(theme.secondary).child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .justify_between()
                                .child(
                                    div()
                                        .child(
                                            div().child(name).text_2xl().text_color(theme.primary),
                                        )
                                        .child(format!(
                                            "{} • {}",
                                            current_version,
                                            codename.unwrap_or_default()
                                        )),
                                )
                                .child(
                                    div()
                                        .child(description.unwrap_or("No description".to_string())),
                                )
                                .child(
                                    div().child(format!("Status : {}", status.as_str())).child(
                                        div()
                                            .flex()
                                            .gap_2()
                                            .child(button(ProjectStatus::Finished))
                                            .child(button(ProjectStatus::Propousing))
                                            .child(button(ProjectStatus::Prospecting))
                                            .child(button(ProjectStatus::Refactoring))
                                            .child(button(ProjectStatus::Started)),
                                    ),
                                ),
                        ),
                    ),
            )
            .child(
                div()
                    .grid()
                    .gap_2()
                    .p_2()
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child(
                                div()
                                    .child("Project history")
                                    .text_lg()
                                    .text_color(theme.primary),
                            )
                            .child(
                                Button::new("toogle-history")
                                    .secondary()
                                    .label("Show/Hide")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.toggle_history(cx);
                                    })),
                            ),
                    )
                    .when(self.show_history, |t| {
                        t.child(
                            div()
                                .border_1()
                                .border_color(theme.border)
                                .h(px(620.))
                                .w_full()
                                .overflow_hidden()
                                .child(List::new(&self.history_list)),
                        )
                    }),
            )
            .child(
                div()
                    .p_2()
                    .child(div().child(format!(
                        "Stakeholders [{}]",
                        self.stakeholders.clone().unwrap_or_default().len()
                    )))
                    .child(div().p_2().child(self.relations_view.clone())),
            )
            .into_any_element()
    }
}
