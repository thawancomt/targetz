use gpui_kit::{
    App, AppContext, Context, IntoElement, ParentElement, Styled,
    component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
    },
    div,
    prelude::FluentBuilder,
};
use shared::customer::Customer;

use crate::project_detail_view::{
    project_relations::state::ProjectRelationView, state::ProjectDetailView,
};

pub fn customer_item(
    customer: Customer,
    cx: &mut Context<ProjectRelationView>,
) -> impl IntoElement {
    let theme = cx.theme();

    div()
        .flex()
        .bg(theme.secondary)
        .justify_between()
        .p_2()
        .border_1()
        .border_color(theme.border)
        .child(
            div()
                .flex()
                .flex_col()
                .p_2()
                .child(div().child(div().child(format!("{}", customer.name))))
                .child(
                    div()
                        .child(
                            div()
                                .child(format!("Created at: {}", customer.created_at))
                                .text_xs(),
                        )
                        .when(!customer.email.is_empty(), |d| {
                            d.child(div().child(format!("email: {}", customer.email)))
                                .text_xs()
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
        )
}
