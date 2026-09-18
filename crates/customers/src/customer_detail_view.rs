use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, FontWeight, IntoElement, ParentElement, Render,
    RenderOnce, Styled, Window,
    base::{StyledExt, v_flex},
    component::{
        ActiveTheme, Icon, IconName, button::Button, chart::LineChart, plot::Grid, tag::Tag,
    },
    div,
};
use shared::{
    customer::{Customer, Persisted},
    db::DbPool,
    events::AppEvent,
};

use crate::{
    customer_repository::CustomerRepository,
    interactions::{
        create_interaction_view::{self, CreateInteractionView},
        interaction_repository::{self, InteractionRepository},
    },
};

pub struct CustomerDetailView {
    customer: Option<Customer<Persisted>>,
    repository: CustomerRepository,
    interaction_repository: InteractionRepository,
    create_interaction_view: Entity<CreateInteractionView>,
}

impl EventEmitter<AppEvent> for CustomerDetailView {}

impl CustomerDetailView {
    pub fn view(
        window: &mut Window,
        cx: &mut App,
        customer: Option<Customer<Persisted>>,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, customer))
    }

    pub fn new(window: &mut Window, cx: &mut App, customer: Option<Customer<Persisted>>) -> Self {
        let pool = cx.global::<DbPool>().0.clone();

        let repository = CustomerRepository::new(pool.clone());

        let interaction_repository = InteractionRepository::new(pool.clone(), customer.clone());

        let create_interaction_view =
            CreateInteractionView::view(window, cx, interaction_repository.clone());

        Self {
            customer,
            repository,
            interaction_repository,
            create_interaction_view,
        }
    }

    pub fn set_customer(&mut self, customer: Customer) {
        self.customer = Some(customer)
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
                    .text_color(cx.theme().primary),
            )
    }
}

impl Render for CustomerDetailView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.customer.clone() {
            Some(customer) => {
                let customer_id = customer.id;
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
                                            .text_color(cx.theme().foreground),
                                    )
                                    .child(
                                        div()
                                            .child(format!("ID #{}", customer.id))
                                            .font_family("Geist Mono")
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground),
                                    )
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
                            ),
                    )
                    .child(
                        v_flex()
                            .w_full()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_md()
                            .child(self.info_row(
                                IconName::Calendar,
                                "Email",
                                Some(customer.email),
                                cx,
                            ))
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
                    .child(
                        div().child(Button::new("Delete customer").label("Delete").on_click(
                            cx.listener(move |view, _e, _window, cx| {
                                let repo = view.repository.clone();
                                cx.spawn(async move |this, cx| {
                                    let _ = match repo.delete_customer(customer_id).await {
                                        Ok(_) => {
                                            let _ = this.update(cx, |_this, cx| {
                                                cx.emit(AppEvent::DeletedCustomer(customer_id));
                                            });
                                        }
                                        Err(_) => {}
                                    };
                                })
                                .detach();
                            }),
                        )),
                    )
                    .child(self.create_interaction_view.clone().into_any_element())
                    .into_any_element()
            }
            None => div().into_any_element(),
        }
    }
}
