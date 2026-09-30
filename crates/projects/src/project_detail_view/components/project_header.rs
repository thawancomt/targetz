use gpui_kit::{
    AnyElement, App, Div, IntoElement, ParentElement, Styled,
    base::{StyledExt, v_flex},
    component::{ActiveTheme, tag::Tag},
    div,
    prelude::FluentBuilder,
};
use shared::{project::Project, ui::caption};

/// Title block: mono caption, name, status tag, description. `actions` sits
/// on the right (edit button).
pub(crate) fn project_header(project: Project, actions: Option<AnyElement>, cx: &mut App) -> Div {
    let theme = cx.theme();

    let Project {
        id,
        name,
        current_version,
        status,
        description,
        codename,
        ..
    } = project;

    let mut caption_text = format!("PROJECT #{id} · v{current_version}");
    if let Some(codename) = codename.filter(|c| !c.is_empty()) {
        caption_text.push_str(&format!(" · {codename}"));
    }

    div()
        .h_flex()
        .justify_between()
        .items_start()
        .gap_4()
        .pb_4()
        .border_b_1()
        .border_color(theme.border)
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_2()
                .child(caption(caption_text, theme))
                .child(div().w_full().child(name).text_3xl().text_color(theme.foreground))
                .child(div().h_flex().child(Tag::secondary().child(status.as_str())))
                .child(
                    div()
                        .child(description.unwrap_or_else(|| "No description".to_string()))
                        .text_color(theme.muted_foreground),
                ),
        )
        .when_some(actions, |d, a| d.child(div().flex_none().child(a)))
}
