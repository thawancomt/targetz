use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Window,
    base::{StyledExt, v_flex},
    component::{
        ActiveTheme, Icon, IconName, Theme, button::Button, scroll::ScrollableElement, tag::Tag,
    },
    div,
    prelude::FluentBuilder,
};
use shared::{
    customer::{Customer, Persisted},
    customer_interaction::Interaction,
    db::DbPool,
    events::AppEvent,
};

use crate::{
    customer_repository::CustomerRepository,
    interactions::{
        create_interaction_view::{CreateInteractionView, CreateInteractionViewEvent},
        interaction_repository::InteractionRepository,
    },
};

pub struct CustomerDetailView {
    customer: Option<Customer<Persisted>>,
    interactions: Option<Vec<Interaction>>,
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

    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        customer: Option<Customer<Persisted>>,
    ) -> Self {
        let pool = cx.global::<DbPool>().0.clone();

        let repository = CustomerRepository::new(pool.clone());

        let interaction_repository = InteractionRepository::new(pool.clone(), customer.clone());

        let create_interaction_view = CreateInteractionView::view(
            window,
            cx,
            customer.clone(),
            interaction_repository.clone(),
        );

        cx.subscribe(&create_interaction_view, |view, _e, event, cx| {
            match event {
                CreateInteractionViewEvent::CreatedNewInteraction(interaction) => {
                    view.add_interaction(interaction.to_owned());
                }
                CreateInteractionViewEvent::DeletedInteraction(deleted_id) => {
                    view.remove_interaction(*deleted_id);
                }
            };
            cx.notify();
        })
        .detach();

        Self {
            customer,
            repository,
            interaction_repository,
            create_interaction_view,
            interactions: None,
        }
    }

    pub fn hydrate_interactions(&mut self, cx: &mut Context<Self>) {
        let repository_handle = self.interaction_repository.clone();
        cx.spawn(async move |view, view_contex| {
            let result = repository_handle.get_interactions_for_customer().await;
            let _ = view.update(view_contex, move |this, _cx| {
                match result {
                    Ok(interactions) => {
                        this.interactions = Some(interactions);
                        println!("found {}", this.interactions.iter().len())
                    }
                    Err(e) => eprintln!("{}", e.to_string()),
                };

                _cx.notify();
            });
        })
        .detach();
    }

    pub fn add_interaction(&mut self, interaction: Interaction) {
        match self.interactions.as_mut() {
            Some(interactions) => {
                interactions.push(interaction);
                println!("Interaction add to the array");
            }
            None => {
                eprintln!(
                    "Tried to ADD an interaction, but the interactions array is not hydrated"
                );
            }
        }
    }

    pub fn remove_interaction(&mut self, id: i64) {
        match self.interactions.as_mut() {
            Some(interactions) => {
                interactions.retain(|interaction| interaction.id != id);
                println!("Interaction {} removed", id);
            }
            None => {
                eprintln!(
                    "Tried to REMOVE an interaction, but the interactions array is not hydrated"
                );
            }
        }
    }

    pub fn set_customer(&mut self, customer: Customer, cx: &mut Context<Self>) {
        self.customer = Some(customer.clone());
        self.create_interaction_view
            .update(cx, |interaction_view, interaction_view_context| {
                interaction_view.customer = Some(customer.clone());
                self.interaction_repository.customer = Some(customer.clone());
                interaction_view.interaction_repository.customer = Some(customer);
                interaction_view_context.notify();
            });
        cx.notify();
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

    pub fn interaction_item(&self, interaction: Interaction, theme: &Theme) -> impl IntoElement {
        let is_no_response = matches!(
            interaction.status,
            shared::customer_interaction::InteractionStatus::NoResponse
        );

        let accent_color = if is_no_response {
            theme.danger
        } else {
            theme.selection
        };

        let status_tag = if is_no_response {
            Tag::danger().child(interaction.status.as_str().to_string())
        } else {
            Tag::success().child(interaction.status.as_str().to_string())
        };

        div()
            .w_full()
            .p_3()
            .gap_2()
            .border_1()
            .border_color(accent_color)
            .bg(theme.accent)
            .flex()
            .flex_col()
            // Linha de cabeçalho: data + tag lado a lado
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .child(match interaction.interaction_date.clone().is_empty() {
                                false => interaction.interaction_date.clone(),
                                true => "Missing interaction date".to_string(),
                            })
                            .text_lg(),
                    )
                    .child(status_tag),
            )
            // Nota, com cor levemente atenuada quando ausente
            .child(
                div()
                    .text_color(theme.selection)
                    .child(match &interaction.note {
                        Some(note) => note.clone(),
                        None => "No note left".to_string(),
                    }),
            )
    }
}

impl Render for CustomerDetailView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.customer.clone() {
            Some(customer) => {
                let customer_id = customer.id;
                v_flex()
                    .id("main-content")
                    .h_full()
                    .overflow_y_scrollbar()
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
                    .child({
                        match self.interactions.as_ref() {
                            Some(interactions) => {
                                let theme = cx.theme();
                                div()
                                    .child(div().child("Recent interactions").text_lg())
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .children(
                                        interactions
                                            .iter()
                                            .map(|ii| self.interaction_item(ii.clone(), theme)),
                                    )
                                    .into_any_element()
                            }
                            None => div().into_any_element(),
                        }
                    })
                    .child(self.create_interaction_view.clone().into_any_element())
                    .into_any_element()
            }
            None => div().into_any_element(),
        }
    }
}
