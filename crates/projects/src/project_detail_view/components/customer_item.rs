use gpui_kit::{
    App, AppContext, Context, IntoElement, ParentElement, Styled,
    component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
    },
    div,
    prelude::FluentBuilder,
};
use shared::{customer::Customer, ui::rail_card};

use crate::project_detail_view::{
    project_relations::state::ProjectRelationView, state::ProjectDetailView,
};

pub fn customer_item(
    customer: Customer,
    cx: &mut Context<ProjectRelationView>,
) -> impl IntoElement {
    let theme = cx.theme();

    rail_card(
        false,
        div()
            .flex()
            .flex_1()
            .min_w_0()
            .justify_between()
            .items_center()
            .p_2()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p_2()
                    .child(
                        div()
                            .child(customer.name.clone())
                            .text_color(theme.foreground),
                    )
                    .child(
                        div()
                            .child(
                                div()
                                    .child(format!("created {}", customer.created_at))
                                    .text_xs()
                                    .text_color(theme.muted_foreground),
                            )
                            .when(!customer.email.is_empty(), |d| {
                                d.child(div().child(format!("{}", customer.email)))
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                            }),
                    ),
            )
            .child(
                Button::new(format!("{}", customer.id))
                    .label("Add")
                    .primary()
                    .on_click(cx.listener(move |this, _click, window, cx| {
                        this.add_stakeholder(customer.clone(), window, cx);
                    })),
            ),
        theme,
    )
}
