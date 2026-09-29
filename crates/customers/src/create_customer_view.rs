use std::collections::HashMap;

use crate::customer_repository::CustomerRepository;
use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, InteractiveElement, IntoElement, ParentElement,
    Render, Styled, Window,
    base::{Disableable, input::InputEvent},
    component::{
        WindowExt,
        button::{Button, ButtonVariants},
        form::Field,
        input::{Input, InputState},
        scroll::ScrollableElement,
        switch::Switch,
    },
    div,
    prelude::FluentBuilder,
    px,
};
use shared::{
    customer::{Customer, Draft},
    db::DbPool,
    theme::AppColors,
};

pub enum CreateCustomerEvent {
    Created(Customer),
    FaileToCreate,
    InputChange,
}

impl EventEmitter<CreateCustomerEvent> for CreateCustomerView {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustomerFormFieldId {
    Name,
    Email,
    Address,
    InstagramUrl,
    SiteUrl,
    PhoneNumber,
}

pub struct FormFieldDescriptor {
    pub id: CustomerFormFieldId,
    pub label: &'static str,
}

pub const TEXT_FIELDS: &[FormFieldDescriptor] = &[
    FormFieldDescriptor {
        id: CustomerFormFieldId::Name,
        label: "Name",
    },
    FormFieldDescriptor {
        id: CustomerFormFieldId::Email,
        label: "Email",
    },
    FormFieldDescriptor {
        id: CustomerFormFieldId::Address,
        label: "Address",
    },
    FormFieldDescriptor {
        id: CustomerFormFieldId::InstagramUrl,
        label: "Instagram",
    },
    FormFieldDescriptor {
        id: CustomerFormFieldId::SiteUrl,
        label: "Site url",
    },
    FormFieldDescriptor {
        id: CustomerFormFieldId::PhoneNumber,
        label: "Phone number",
    },
];

pub struct FormFieldState {
    pub value: String,
    pub input: Entity<InputState>,
}

pub struct CreateCustomerView {
    pub text_fields: HashMap<CustomerFormFieldId, FormFieldState>,

    pub is_client: bool,
    pub contacted: bool,

    pub repository: CustomerRepository,

    pub was_edited: bool,
}

fn field(label: String, input: &Entity<InputState>) -> impl IntoElement {
    Field::new().label(label).child(Input::new(input))
}

fn switch_field(label: String, child: impl IntoElement) -> impl IntoElement {
    Field::new().label(label).p_2().w_auto().child(child)
}

impl Render for CreateCustomerView {
    fn render(
        &mut self,
        window: &mut gpui_kit::Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let wide = window.viewport_size().width >= px(768. + 300.);

        div()
            .p_2()
            .flex()
            .flex_col()
            .gap_2()
            .overflow_y_scrollbar()
            .child(
                div()
                    .grid()
                    .grid_cols(if wide { 2 } else { 1 })
                    .gap_2()
                    .children(TEXT_FIELDS.iter().map(|desc| {
                        let input = &self.text_fields.get(&desc.id).unwrap().input;
                        field(desc.label.to_string(), input)
                    }))
                    .child(
                        div()
                            .col_span_full()
                            .flex()
                            .w_full()
                            .gap_2()
                            .mt_2()
                            .child(switch_field(
                                "Have been contacted?".to_string(),
                                Switch::new("contacted").checked(self.contacted).on_change(
                                    cx.listener(|this, value, _window, cx| {
                                        this.set_contacted(*value);
                                        cx.notify();
                                    }),
                                ),
                            ))
                            .child(switch_field(
                                "Is already client?".to_string(),
                                Switch::new("is_client").checked(self.is_client).on_change(
                                    cx.listener(|this, value, _, cx| {
                                        this.set_is_client(*value);
                                        cx.notify();
                                    }),
                                ),
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .mt_3()
                    .child(
                        Button::new("reset-form")
                            .ghost()
                            .label("Reset")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.reset_form(window, cx);
                                window.push_notification("Form reset", cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("create customer")
                            .primary()
                            .disabled(!self.was_edited)
                            .label("Save customer")
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.save_customer(cx);
                                cx.notify();
                            })),
                    ),
            )
    }
}

impl CreateCustomerView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut text_fields = HashMap::new();

        for desc in TEXT_FIELDS {
            let input = cx.new(|cx| InputState::new(window, cx));
            let field_id = desc.id;

            cx.subscribe(&input, move |this, input, _event: &InputEvent, cx| {
                cx.emit(CreateCustomerEvent::InputChange);
                if let Some(field) = this.text_fields.get_mut(&field_id) {
                    field.value = input.read(cx).text().to_string();
                    this.was_edited = true;
                }
                cx.notify();
            })
            .detach();

            text_fields.insert(
                desc.id,
                FormFieldState {
                    value: String::new(),
                    input,
                },
            );
        }

        let pool = cx.global::<DbPool>().0.clone();

        let repository = CustomerRepository::new(pool);

        let _ = cx.subscribe_self(|this, event, _cx| match event {
            CreateCustomerEvent::InputChange => this.was_edited = true,
            _ => {}
        });

        Self {
            text_fields,
            is_client: false,
            contacted: false,
            repository,
            was_edited: false,
        }
    }

    pub fn get_value(&self, id: CustomerFormFieldId) -> &str {
        self.text_fields
            .get(&id)
            .map(|f| f.value.as_str())
            .unwrap_or("")
    }

    pub fn set_is_client(&mut self, value: bool) {
        self.is_client = value;

        // if is already a client for the logic its already been contacted
        if value {
            self.contacted = value
        }
    }
    pub fn set_contacted(&mut self, value: bool) {
        self.contacted = value
    }
    pub fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for field in self.text_fields.values_mut() {
            field.value.clear();
            field.input.update(cx, |input, cx| {
                input.set_value("", window, cx);
            });
        }
        self.is_client = false;
        self.contacted = false;
        self.was_edited = false;

        cx.notify();
    }

    pub fn save_customer(&mut self, cx: &mut Context<Self>) {
        let opt_str = |s: &str| {
            if s.trim().is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        };

        let customer = Customer::<Draft> {
            id: Draft,
            name: self.get_value(CustomerFormFieldId::Name).to_string(),
            email: self.get_value(CustomerFormFieldId::Email).to_string(),
            phone_number: self.get_value(CustomerFormFieldId::PhoneNumber).to_string(),
            address: opt_str(self.get_value(CustomerFormFieldId::Address)),
            instagram_url: opt_str(self.get_value(CustomerFormFieldId::InstagramUrl)),
            site_url: opt_str(self.get_value(CustomerFormFieldId::SiteUrl)),
            is_client: self.is_client,
            contacted: self.contacted,
            created_at: String::new(),
        };

        let repository = self.repository.clone();

        cx.spawn(async move |this, cx| {
            let result = repository.create_customer(customer).await;

            this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(new_customer) => {
                        window.push_notification(
                            format!("Customer {} created", new_customer.name),
                            cx,
                        );
                        cx.emit(CreateCustomerEvent::Created(new_customer));
                        this.reset_form(window, cx);
                    }
                    Err(e) => {
                        window.push_notification(format!("Error: {e}"), cx);
                        eprintln!("{}", e.to_string());
                    }
                };
                cx.notify();
            })
        })
        .detach();
    }
}
