use gpui_kit::{
    Context, IntoElement, ParentElement, Render, Styled, base::v_flex, component::ActiveTheme,
    div, prelude::FluentBuilder,
};
use shared::{customer::Customer, ui::section_frame};

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

        let empty = |message: &'static str| {
            div()
                .w_full()
                .p_3()
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(message)
        };

        let stakeholder_count = stakeholders.len();
        let stakeholders_body = v_flex()
            .w_full()
            .gap_2()
            .when(stakeholders.is_empty(), |d| {
                d.child(empty("No stakeholders added"))
            })
            .children(stakeholders);

        let customer_count = customer_views.len();
        let customers_body = v_flex()
            .w_full()
            .gap_2()
            .when(customer_views.is_empty(), |d| {
                d.child(empty("No customers available to add"))
            })
            .children(customer_views);

        v_flex()
            .w_full()
            .gap_6()
            .child(section_frame(
                "Stakeholders",
                Some(stakeholder_count),
                None,
                stakeholders_body,
                &theme,
            ))
            .child(section_frame(
                "Available customers",
                Some(customer_count),
                None,
                customers_body,
                &theme,
            ))
    }
}
