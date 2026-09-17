use gpui_kit::{
    FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window,
    base::{StyledExt, v_flex},
    component::{ActiveTheme, Icon, IconName, tag::Tag},
    div,
};
use shared::customer::{Customer, Persisted};

#[derive(IntoElement)]
pub struct CustomerDetailView {
    customer: Customer<Persisted>,
}

impl CustomerDetailView {
    pub fn new(customer: Customer<Persisted>) -> Self {
        Self { customer }
    }

    fn info_row(
        &self,
        icon: IconName,
        label: &'static str,
        value: Option<String>,
        cx: &gpui_kit::App,
    ) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .p_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(Icon::new(icon).text_color(cx.theme().muted_foreground))
                    .child(
                        div()
                            .child(label)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(cx.theme().foreground),
                    ),
            )
            .child(
                div()
                    .child(value.unwrap_or_else(|| "N/A".to_string()))
                    .font_family("Geist Mono")
                    .text_color(cx.theme().primary),
            )
    }
}

impl RenderOnce for CustomerDetailView {
    fn render(self, _window: &mut Window, cx: &mut gpui_kit::App) -> impl IntoElement {
        let customer = self.customer.clone();

        v_flex()
            .size_full()
            .p_6()
            .gap_6()
            .bg(cx.theme().background)
            // Header / Perfil
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_start()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .pb_4()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .child(customer.name.clone())
                                    .text_3xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground),
                            )
                            .child(
                                div()
                                    .child(format!("ID #{}", customer.id))
                                    .font_family("Geist Mono")
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            ),
                    )
                    // Status Badges
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(Tag::secondary().child(if customer.is_client {
                                "Active Client"
                            } else {
                                "Prospect"
                            }))
                            .child(Tag::secondary().child(if customer.contacted {
                                "Contacted"
                            } else {
                                "Pending Contact"
                            })),
                    ),
            )
            // Tabela de Detalhes
            .child(
                v_flex()
                    .w_full()
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_md()
                    .child(self.info_row(IconName::Calendar, "Email", Some(customer.email), cx))
                    .child(self.info_row(
                        IconName::PanelRight,
                        "Phone Number",
                        Some(customer.phone_number),
                        cx,
                    ))
                    .child(self.info_row(IconName::Map, "Address", customer.address, cx))
                    .child(self.info_row(IconName::Globe, "Website", customer.site_url, cx))
                    .child(self.info_row(
                        IconName::ExternalLink,
                        "Instagram",
                        customer.instagram_url,
                        cx,
                    )),
            )
    }
}
