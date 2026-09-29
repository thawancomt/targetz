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

use crate::project_detail_view::{
    components::project_header::project_header, state::ProjectDetailView,
};

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

        let header = project_header(project.clone(), cx);
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

        let show_save = self.relations_view.read(cx).initial_stakeholders
            != self.relations_view.read(cx).stakeholders;

        div()
            .size_full()
            .h_full()
            .overflow_y_scrollbar()
            .child(header)
            .child(
                div()
                    .m_2()
                    .p_2()
                    .border_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child(format!("Status [{}]", status.as_str()))
                            .child(Button::new("edit-project").label("Edit").on_click(
                                cx.listener(|this, _, window, cx| {
                                    this.open_edit_dialog(window, cx);
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(button(ProjectStatus::Finished))
                            .child(button(ProjectStatus::Propousing))
                            .child(button(ProjectStatus::Prospecting))
                            .child(button(ProjectStatus::Refactoring))
                            .child(button(ProjectStatus::Started)),
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
            .child(div().p_2().child(div().child(self.relations_view.clone())))
            .when(show_save, |d| {
                d.child(
                    div().p_2().child(
                        Button::new("save-button")
                            .primary()
                            .label("Save changes")
                            .on_click(cx.listener(|this, _click, window, cx| {
                                this.save_relations(cx);
                            })),
                    ),
                )
                .p_2()
            })
            .into_any_element()
    }
}
