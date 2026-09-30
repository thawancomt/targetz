//! Small raw-style building blocks shared by the detail views: mono captions,
//! hairline-bordered fact cells and titled section frames.
use gpui_kit::{
    AnyElement, Div, FontWeight, IntoElement, ParentElement, Styled,
    base::{StyledExt, v_flex},
    component::Theme,
    div,
    prelude::FluentBuilder,
};

/// Small uppercase mono caption.
pub fn caption(text: impl AsRef<str>, theme: &Theme) -> Div {
    div()
        .child(text.as_ref().to_uppercase())
        .font_family("Geist Mono")
        .text_xs()
        .text_color(theme.muted_foreground)
}

/// One cell of a fact grid: caption over value, `—` when missing.
/// Put cells in a row inside a container with `border_t_1` and `border_l_1`.
pub fn fact_cell(label: &'static str, value: Option<String>, theme: &Theme) -> impl IntoElement {
    let missing = value.is_none();
    v_flex()
        .flex_1()
        .min_w_0()
        .p_3()
        .gap_1()
        .border_r_1()
        .border_b_1()
        .border_color(theme.border)
        .child(caption(label, theme))
        .child(
            div()
                .min_w_0()
                .overflow_hidden()
                .child(value.unwrap_or_else(|| "—".to_string()))
                .font_weight(FontWeight::MEDIUM)
                .text_color(if missing {
                    theme.muted_foreground
                } else {
                    theme.foreground
                }),
        )
}

/// Bordered frame with a muted title bar (caption, optional count, optional
/// action on the right) over the body.
pub fn section_frame(
    title: impl AsRef<str>,
    count: Option<usize>,
    action: Option<AnyElement>,
    body: impl IntoElement,
    theme: &Theme,
) -> Div {
    div()
        .v_flex()
        .w_full()
        .min_w_0()
        .border_1()
        .border_color(theme.border)
        .child(
            div()
                .h_flex()
                .justify_between()
                .items_center()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(theme.border)
                .bg(theme.muted)
                .child(
                    div()
                        .h_flex()
                        .gap_3()
                        .items_center()
                        .child(caption(title, theme))
                        .when_some(count, |d, n| {
                            d.child(
                                div()
                                    .child(format!("{n:02}"))
                                    .font_family("Geist Mono")
                                    .text_xs()
                                    .text_color(theme.primary),
                            )
                        }),
                )
                .when_some(action, |d, a| d.child(a)),
        )
        .child(v_flex().w_full().min_w_0().p_3().gap_2().child(body))
}

/// List-item frame shared by customer, project and document cards: hairline
/// border, primary left rail, accent-filled body. `selected` tints the
/// body and outlines it in the primary colour. Add `.id(..)`/hover on the result.
pub fn rail_card(selected: bool, body: impl IntoElement, theme: &Theme) -> Div {
    div()
        .h_flex()
        .w_full()
        .overflow_hidden()
        .border_1()
        .rounded(theme.radius)
        .border_color(if selected {
            theme.primary
        } else {
            theme.border
        })
        .bg(if selected {
            theme.selection
        } else {
            theme.accent
        })
        .child(div().w(gpui_kit::px(3.)).flex_none().bg(theme.primary))
        .child(body)
}
