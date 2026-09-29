use gpui_kit::{
    App, Context, Div, IntoElement, ParentElement, Styled,
    component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
        scroll::ScrollableElement,
    },
    div,
    prelude::FluentBuilder,
};
use shared::project::{Project, ProjectStatus};

use crate::project_detail_view::state::ProjectDetailView;

pub(crate) fn project_header(project: Project, cx: &mut App) -> Div {
    let theme = cx.theme();

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
                            .child(div().child(name).text_2xl().text_color(theme.primary))
                            .child(format!(
                                "{} • {}",
                                current_version,
                                codename.unwrap_or_default()
                            )),
                    )
                    .child(div().child(description.unwrap_or("No description".to_string()))),
            ),
        )
}
