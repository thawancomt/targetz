use gpui_kit::{
    AnyView, App, AppContext, Context, Entity, EventEmitter, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Window,
    base::v_flex,
    component::{
        ActiveTheme, Icon, IconName, Theme, WindowExt,
        button::{Button, ButtonVariants},
        scroll::ScrollableElement,
        tag::Tag,
    },
    div, px,
    prelude::FluentBuilder,
};
use shared::{
    customer::{Customer, Persisted},
    customer_interaction::Interaction,
    db::DbPool,
    events::AppEvent,
    ui::{fact_cell, section_frame},
};

use crate::{
    customer_repository::CustomerRepository,
    edit_customer_view::{events::CustomerUpdateEvent, state::CustomerUpdateView},
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
    pub edit_view: Entity<CustomerUpdateView>,
    /// Projects and documents sections (stacked at the end). Filled by the app shell so this crate
    /// does not depend on `projects` or `documents`.
    extra_sections: Vec<AnyView>,
    on_customer_changed: Option<Box<dyn Fn(i64, &mut App) + 'static>>,
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

        let edit_view = CustomerUpdateView::view(window, cx, customer.clone());

        cx.subscribe(&edit_view, |this, _emitter, event, cx| match event {
            CustomerUpdateEvent::UpdatedCustomer(updated) => {
                this.set_customer(updated.clone(), cx);
                cx.emit(AppEvent::UpdatedCustomer(updated.clone()));
                cx.notify();
            }
        })
        .detach();

        Self {
            customer,
            repository,
            interaction_repository,
            create_interaction_view,
            edit_view,
            interactions: None,
            extra_sections: Vec::new(),
            on_customer_changed: None,
        }
    }

    /// Installs the sections shown below the interactions and the hook that
    /// reloads them when the open customer changes. The hook is owned by the
    /// app shell.
    pub fn set_extra_sections(
        &mut self,
        sections: Vec<AnyView>,
        on_customer_changed: impl Fn(i64, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        self.extra_sections = sections;
        self.on_customer_changed = Some(Box::new(on_customer_changed));
        if let Some(customer) = self.customer.as_ref() {
            if let Some(hook) = self.on_customer_changed.as_ref() {
                hook(customer.id, cx);
            }
        }
        cx.notify();
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
        if let Some(hook) = self.on_customer_changed.as_ref() {
            hook(customer.id, cx);
        }
        self.customer = Some(customer.clone());
        self.create_interaction_view
            .update(cx, |interaction_view, interaction_view_context| {
                interaction_view.customer = Some(customer.clone());
                self.interaction_repository.customer = Some(customer.clone());
                interaction_view.interaction_repository.customer = Some(customer.clone());
                interaction_view_context.notify();
            });
        self.edit_view.update(cx, |edit, _cx| {
            edit.customer = Some(customer);
        });
        cx.notify();
    }

    pub fn open_edit_dialog(&self, window: &mut Window, cx: &mut Context<Self>) {
        let view = self.edit_view.clone();
        if let Some(c) = self.customer.clone() {
            view.update(cx, |edit, cx| {
                edit.set_customer(c, window, cx);
            });
        }

        window.open_dialog(cx, move |dialog, _window, _cx| {
            dialog
                .title("Edit Customer")
                .child(view.clone())
                .w(px(560.))
        });
    }

    fn fact_cell(
        &self,
        label: &'static str,
        value: Option<String>,
        cx: &gpui_kit::App,
    ) -> impl IntoElement {
        fact_cell(label, value, cx.theme())
    }

    fn section(
        &self,
        title: &'static str,
        count: Option<usize>,
        body: impl IntoElement,
        theme: &Theme,
    ) -> impl IntoElement {
        section_frame(title, count, None, body, theme)
    }

    pub fn interaction_item(&self, interaction: Interaction, theme: &Theme) -> impl IntoElement {
        let is_no_response = matches!(
            interaction.status,
            shared::customer_interaction::InteractionStatus::NoResponse
        );

        let status_tag = if is_no_response {
            Tag::info().child(interaction.status.as_str().to_string())
        } else {
            Tag::warning().child(interaction.status.as_str().to_string())
        };

        let date = if interaction.interaction_date.is_empty() {
            "missing date".to_string()
        } else {
            interaction.interaction_date.clone()
        };
        let (note, has_note) = match &interaction.note {
            Some(note) => (note.replace('\n', " "), true),
            None => ("No note left".to_string(), false),
        };

        // Left rail in the accent colour, like a log line.
        div()
            .flex()
            .w_full()
            .min_w_0()
            .overflow_hidden()
            .border_1()
            .border_color(theme.border)
            .child(div().w(px(3.)).flex_none().bg(if is_no_response {
                theme.muted_foreground
            } else {
                theme.primary
            }))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .p_3()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .child(date)
                                    .font_family("Geist Mono")
                                    .text_sm()
                                    .text_color(theme.foreground),
                            )
                            .child(status_tag),
                    )
                    .child(div().w_full().min_w_0().overflow_hidden().child(note).text_color(
                        if has_note {
                            theme.foreground
                        } else {
                            theme.muted_foreground
                        },
                    )),
            )
    }
}

impl Render for CustomerDetailView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(customer) = self.customer.clone() else {
            return div().into_any_element();
        };
        let customer_id = customer.id;
        let theme = cx.theme().clone();

        let interactions_body = match self.interactions.as_ref() {
            Some(interactions) if !interactions.is_empty() => v_flex()
                .w_full()
                .min_w_0()
                .gap_2()
                .children(
                    interactions
                        .iter()
                        .map(|ii| self.interaction_item(ii.clone(), &theme)),
                )
                .into_any_element(),
            Some(_) => div()
                .child("No interactions yet.")
                .text_sm()
                .text_color(theme.muted_foreground)
                .into_any_element(),
            None => div()
                .child("Loading…")
                .text_sm()
                .text_color(theme.muted_foreground)
                .into_any_element(),
        };
        let interaction_count = self.interactions.as_ref().map(|i| i.len());

        let header = div()
            .flex()
            .justify_between()
            .items_start()
            .gap_4()
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_2()
                    .child(
                        div()
                            .child(format!("CUSTOMER #{}", customer.id))
                            .font_family("Geist Mono")
                            .text_xs()
                            .text_color(theme.muted_foreground),
                    )
                    .child(
                        div()
                            .child(customer.name.clone())
                            .text_3xl()
                            .text_color(theme.foreground),
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
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .flex_none()
                    .child(
                        Button::new("edit-customer")
                            .secondary()
                            .label("Edit")
                            .on_click(cx.listener(|this, _e, window, cx| {
                                this.open_edit_dialog(window, cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("delete-customer")
                            .danger()
                            .label("Delete")
                            .on_click(cx.listener(move |view, _e, _window, cx| {
                                let repo = view.repository.clone();
                                cx.spawn(async move |this, cx| {
                                    if repo.delete_customer(customer_id).await.is_ok() {
                                        let _ = this.update(cx, |_this, cx| {
                                            cx.emit(AppEvent::DeletedCustomer(customer_id));
                                        });
                                    }
                                })
                                .detach();
                            })),
                    ),
            );

        // Contact grid: two cells per row, hairline borders.
        let contact = v_flex()
            .w_full()
            .border_t_1()
            .border_l_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .child(self.fact_cell("Email", Some(customer.email.clone()), cx))
                    .child(self.fact_cell("Phone", Some(customer.phone_number.clone()), cx)),
            )
            .child(
                div()
                    .flex()
                    .child(self.fact_cell("Website", customer.site_url.clone(), cx))
                    .child(self.fact_cell("Instagram", customer.instagram_url.clone(), cx)),
            )
            .child(
                div()
                    .flex()
                    .child(self.fact_cell("Address", customer.address.clone(), cx)),
            );

        v_flex()
            .id("main-content")
            .h_full()
            .overflow_y_scrollbar()
            .size_full()
            .min_w_0()
            .p_6()
            .gap_6()
            .bg(theme.background)
            .child(
                div()
                    .pb_4()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(header),
            )
            .child(contact)
            .child(self.section("Interactions", interaction_count, interactions_body, &theme))
            .child(self.create_interaction_view.clone())
            .children(self.extra_sections.iter().cloned())
            .into_any_element()
    }
}
