use gpui_kit::{
    Context, IntoElement, ParentElement, Render, Styled,
    base::v_flex,
    component::{ActiveTheme, button::Button},
    div,
    prelude::FluentBuilder,
};
use shared::customer::Customer;

use crate::project_detail_view::{
    components::customer_item::customer_item, project_relations::state::ProjectRelationView,
};

impl Render for ProjectRelationView {
    fn render(
        &mut self,
        window: &mut gpui_kit::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let theme = cx.theme();

        let current_stakeholders = self.stakeholders.clone().unwrap_or_default().clone();

        let customers: Vec<Customer> = self
            .customers
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|c| !current_stakeholders.iter().find(|s| s.id == c.id).is_some())
            .collect();

        let stakeholders = self.stakeholders_views.clone().unwrap_or_default();

        v_flex()
            .child(
                div()
                    .child(div().child("Relations"))
                    .when(stakeholders.len() == 0, |d| {
                        d.child(
                            div()
                                .border_1()
                                .border_color(theme.border)
                                .flex_1()
                                .justify_center()
                                .items_center()
                                .p_2()
                                .child("No customer to add"),
                        )
                    })
                    .when(stakeholders.len() > 0, |d| {
                        d.child(
                            div()
                                .flex_1()
                                .p_2()
                                .border_1()
                                .border_color(theme.border)
                                .children(stakeholders)
                                .grid()
                                .gap_2(),
                        )
                    }),
            )
            .child(
                div().child(div().child("Customers")).child(
                    div()
                        .p_2()
                        .border_1()
                        .border_color(theme.border)
                        .flex_1()
                        .when(customers.len() == 0, |d| {
                            d.child(
                                div()
                                    .flex_1()
                                    .justify_center()
                                    .items_center()
                                    .p_2()
                                    .child("No customer to add"),
                            )
                        })
                        .children(
                            customers
                                .iter()
                                .map(|c| customer_item(c.clone(), cx).into_any_element()),
                        )
                        .grid()
                        .gap_2(),
                ),
            )
    }
}
