use gpui_kit::{
    InteractiveElement, IntoElement, ParentElement, Render, Styled,
    base::{StyledExt, v_flex},
    component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
        scroll::ScrollableElement,
    },
    div,
    prelude::FluentBuilder,
    px,
};
use shared::{
    project::{Project, ProjectStatus},
    ui::{fact_cell, section_frame},
};

use crate::project_detail_view::{
    components::{history_item::history_item, project_header::project_header}, state::ProjectDetailView,
};

impl Render for ProjectDetailView {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let Some(project) = self.project.clone() else {
            return div().into_any_element();
        };
        let Project {
            status,
            start_date,
            target_deadline,
            budget,
            site_url,
            created_at,
            updated_at,
            current_version,
            codename,
            ..
        } = project.clone();

        let theme = cx.theme().clone();

        let edit_button = Button::new("edit-project")
            .secondary()
            .label("Edit")
            .on_click(cx.listener(|this, _, window, cx| {
                this.open_edit_dialog(window, cx);
                cx.notify();
            }))
            .into_any_element();
        let header = project_header(project, Some(edit_button), cx);

        let status_button = |project_status: ProjectStatus| {
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

        let facts = v_flex()
            .w_full()
            .border_t_1()
            .border_l_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .child(fact_cell("Version", Some(current_version), &theme))
                    .child(fact_cell("Codename", codename, &theme))
                    .child(fact_cell("Budget", budget.map(|b| format!("{b:.2}")), &theme)),
            )
            .child(
                div()
                    .flex()
                    .child(fact_cell("Start", start_date, &theme))
                    .child(fact_cell("Deadline", target_deadline, &theme))
                    .child(fact_cell("Website", site_url, &theme)),
            )
            .child(
                div()
                    .flex()
                    .child(fact_cell("Created", Some(created_at), &theme))
                    .child(fact_cell("Updated", updated_at, &theme)),
            );

        let status_section = section_frame(
            "Status",
            None,
            None,
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .child(status_button(ProjectStatus::Prospecting))
                .child(status_button(ProjectStatus::Propousing))
                .child(status_button(ProjectStatus::Started))
                .child(status_button(ProjectStatus::Refactoring))
                .child(status_button(ProjectStatus::Finished)),
            &theme,
        );

        let history = self.history.clone().unwrap_or_default();
        let history_count = history.len();
        let last = history_count.saturating_sub(1);
        // Newest first.
        let history_body = v_flex()
            .w_full()
            .when(history.is_empty(), |d| {
                d.child(
                    div()
                        .p_3()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("No status changes yet."),
                )
            })
            .children(
                history
                    .into_iter()
                    .rev()
                    .enumerate()
                    .map(|(ix, row)| history_item(row, ix == last, &theme)),
            );
        let history_section = section_frame(
            "History",
            Some(history_count),
            None,
            history_body,
            &theme,
        );

        let main = v_flex()
            .flex_1()
            .min_w_0()
            .gap_6()
            .child(facts)
            .child(status_section)
            .child(self.relations_view.clone())
            .when_some(self.documents_section(), |parent, section| {
                parent.child(section)
            });

        let side = v_flex().w(px(340.)).flex_none().child(history_section);

        let content = v_flex()
            .id("project-detail")
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_y_scrollbar()
            .p_6()
            .gap_6()
            .child(header)
            .child(div().flex().items_start().gap_6().child(main).child(side));

        // Pinned below the scroll area so it stays visible.
        v_flex()
            .size_full()
            .min_w_0()
            .bg(theme.background)
            .child(content)
            .when(show_save, |d| {
                d.child(
                    div()
                        .h_flex()
                        .flex_none()
                        .justify_between()
                        .items_center()
                        .px_6()
                        .py_3()
                        .border_t_1()
                        .border_color(theme.primary)
                        .bg(theme.accent)
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child("Unsaved stakeholder changes."),
                        )
                        .child(
                            Button::new("save-button")
                                .primary()
                                .label("Save changes")
                                .on_click(cx.listener(|this, _click, _window, cx| {
                                    this.save_relations(cx);
                                })),
                        ),
                )
            })
            .into_any_element()
    }
}
