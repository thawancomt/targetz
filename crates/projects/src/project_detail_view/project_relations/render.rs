use gpui_kit::{
    Context, IntoElement, ParentElement, Render, Styled, base::v_flex, component::ActiveTheme, div,
    prelude::FluentBuilder,
};
use shared::customer::Customer;

use crate::project_detail_view::{
    components::customer_item::customer_item, project_relations::state::ProjectRelationView,
};

impl Render for ProjectRelationView {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let theme = cx.theme().clone();

        let current_stakeholders = self.stakeholders.clone().unwrap_or_default();

        let customers: Vec<Customer> = self
            .customers
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|c| !current_stakeholders.iter().any(|s| s.id == c.id))
            .collect();

        let stakeholders = self.stakeholders_views.clone().unwrap_or_default();

        let customer_views: Vec<_> = customers
            .into_iter()
            .map(|c| customer_item(c, cx).into_any_element())
            .collect();

        v_flex()
            .w_full()
            .gap_4()
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .child(format!("Stakeholders [{}]", stakeholders.len()))
                            .text_lg()
                            .text_color(theme.primary),
                    )
                    .when(stakeholders.is_empty(), |d| {
                        d.child(
                            div()
                                .w_full()
                                .p_4()
                                .border_1()
                                .border_color(theme.border)
                                .rounded_md()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child("No stakeholders added"),
                        )
                    })
                    .when(!stakeholders.is_empty(), |d| {
                        d.child(
                            div()
                                .w_full()
                                .p_2()
                                .border_1()
                                .border_color(theme.border)
                                .rounded_md()
                                .grid()
                                .gap_2()
                                .children(stakeholders),
                        )
                    }),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .child(format!("Customers [{}]", customer_views.len()))
                            .text_lg()
                            .text_color(theme.primary),
                    )
                    .when(customer_views.is_empty(), |d| {
                        d.child(
                            div()
                                .w_full()
                                .p_4()
                                .border_1()
                                .border_color(theme.border)
                                .rounded_md()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child("No customers available to add"),
                        )
                    })
                    .when(!customer_views.is_empty(), |d| {
                        d.child(
                            div()
                                .w_full()
                                .p_2()
                                .border_1()
                                .border_color(theme.border)
                                .rounded_md()
                                .grid()
                                .gap_2()
                                .children(customer_views),
                        )
                    }),
            )
    }
}
