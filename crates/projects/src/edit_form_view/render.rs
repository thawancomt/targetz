use gpui_kit::{
    ParentElement, Render, Styled,
    component::{
        WindowExt,
        button::{Button, ButtonVariants},
    },
    div, px,
};
use shared::{
    form_utils::{app_date_field, app_field, date_input, text_input},
    project::Project,
};

use crate::{
    edit_form_view::state::ProjectUpdateView,
    project_detail_view::components::project_header::project_header,
};

impl Render for ProjectUpdateView {
    fn render(
        &mut self,
        window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let Some(project) = self.project_draft.clone() else {
            return div().child("no project set");
        };

        let header = project_header(
            Project {
                id: project.id,
                name: text_input(&self.name, cx).unwrap_or("".to_string()),
                current_version: text_input(&self.version, cx).unwrap_or("".to_string()),
                created_at: project.created_at,
                status: project.status,
                description: text_input(&self.description, cx),
                start_date: date_input(&self.start_date, cx),
                site_url: text_input(&self.site_url, cx),
                codename: text_input(&self.codename, cx),
                target_deadline: date_input(&self.target_deadline, cx),
                updated_at: project.updated_at,
                budget: project.budget,
            },
            cx,
        );

        let wide = window.viewport_size().width >= px(768. + 300.);

        div().child(header).child("Edit it").child(
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
                    Button::new("Save")
                        .primary()
                        .col_span_full()
                        .label("Save project")
                        .on_click(cx.listener(|this, _, window, cx| {
                            window.push_notification("Updating project", cx);
                            let _ = this.update_project(cx);
                        }))
                        .mt_2(),
                ),
        )
    }
}
