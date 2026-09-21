use gpui_kit::{
    App, AppContext, Context, ParentElement, Render, Styled,
    component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
        scroll::ScrollableElement,
    },
    div,
    prelude::FluentBuilder,
};

use crate::{
    project_detail_view::state::ProjectDetailView, projects_list_view::state::ProjectsListView,
};

impl Render for ProjectsListView {
    fn render(
        &mut self,
        window: &mut gpui_kit::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let theme = cx.theme().clone();

        let views = self.project_views.clone();

        let has_open_projects = self.open_projects.len() > 0;
        let has_active_project = self.active_project.is_some();
        let active_p = self.active_project.clone();

        let active_project_view = if has_active_project {
            Some(ProjectDetailView::new(
                window,
                cx,
                active_p.unwrap().clone(),
            ))
        } else {
            None
        };

        div()
            .size_full()
            .min_h_0()
            .p_2()
            .overflow_y_scrollbar()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(format!("Project list view {}", self.projects.len()))
                    .text_lg()
                    .text_color(theme.primary)
                    .child(
                        div()
                            .child(
                                Button::new("open-create-project-dialog")
                                    .label("Create dialog")
                                    .secondary()
                                    .on_click(cx.listener(|this, _e, window, cx| {
                                        this.open_create_project_dilaog(window, cx);
                                    })),
                            )
                            .py_2(),
                    ),
            )
            .when(has_active_project, |t| {
                t.child(active_project_view.unwrap())
            })
            .when(!has_active_project, |t| {
                t.child(div().grid().gap_2().children(views))
            })
    }
}
