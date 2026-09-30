use gpui_kit::{
    IntoElement, ParentElement, Styled,
    base::{StyledExt, v_flex},
    component::{Theme, tag::Tag},
    div,
    prelude::FluentBuilder,
    px,
};
use shared::{project::ProjectHistoryRow, ui::caption};

/// One timeline entry: rail with a dot, timestamp, `from -> to`, then only
/// the details that were actually recorded.
pub(crate) fn history_item(row: ProjectHistoryRow, is_last: bool, theme: &Theme) -> impl IntoElement {
    let ProjectHistoryRow {
        from_status,
        to_status,
        note,
        change_ask_by,
        reason,
        impact_on_target_deadline,
        created_at,
        ..
    } = row;

    let detail = |label: &'static str, value: String| {
        div()
            .h_flex()
            .gap_2()
            .text_sm()
            .child(caption(label, theme).flex_none().w(px(64.)))
            .child(div().min_w_0().child(value).text_color(theme.foreground))
    };

    div()
        .h_flex()
        .items_stretch()
        .gap_3()
        .w_full()
        .child(
            v_flex()
                .items_center()
                .flex_none()
                .w(px(9.))
                .child(div().mt_1().size(px(9.)).bg(theme.primary))
                .when(!is_last, |d| {
                    d.child(div().flex_1().w(px(1.)).bg(theme.border))
                }),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_2()
                .pb_5()
                .child(caption(created_at, theme))
                .child(
                    div()
                        .h_flex()
                        .gap_2()
                        .items_center()
                        .child(Tag::secondary().child(from_status))
                        .child(div().child("→").text_color(theme.muted_foreground))
                        .child(Tag::info().child(to_status)),
                )
                .when_some(impact_on_target_deadline, |d, impact| {
                    d.child(detail("Impact", impact))
                })
                .when_some(change_ask_by, |d, by| d.child(detail("Asked by", by)))
                .when_some(reason, |d, reason| d.child(detail("Reason", reason)))
                .when_some(note, |d, note| {
                    d.child(
                        div()
                            .p_2()
                            .border_1()
                            .border_color(theme.border)
                            .text_sm()
                            .child(note),
                    )
                }),
        )
}
