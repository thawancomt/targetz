use gpui_kit::{
    Context, ParentElement, Render, Styled,
    component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
        scroll::ScrollableElement,
    },
    div,
};

use crate::projects_list_view::state::ProjectsListView;

impl Render for ProjectsListView {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let theme = cx.theme().clone();
        let views = self.project_views.clone();

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
            .child(div().grid().gap_2().children(views))
    }
}
