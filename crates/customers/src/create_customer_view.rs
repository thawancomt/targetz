use crate::customer_repository::CustomerRepository;
use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, InteractiveElement, IntoElement, ParentElement,
    Render, Styled, Window,
    base::{StyledExt, input::InputEvent},
    component::{
        ActiveTheme, WindowExt,
        button::{Button, ButtonVariants},
        form::Field,
        input::{Input, InputState},
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
    Created,
    FaileToCreate,
}

impl EventEmitter<CreateCustomerEvent> for CreateCustomerView {}

pub struct CreateCustomerView {
    pub name: String,
    pub name_input: Entity<InputState>,

    pub instagram_url: String,
    pub instagram_url_input: Entity<InputState>,

    pub site_url: String,
    pub site_url_input: Entity<InputState>,

    pub email: String,
    pub email_input: Entity<InputState>,

    pub address: String,
    pub address_input: Entity<InputState>,

    pub phone_number: String,
    pub phone_number_input: Entity<InputState>,

    pub is_client: bool,
    pub contacted: bool,

    pub repository: CustomerRepository,
}

fn field(label: String, input: &Entity<InputState>) -> impl IntoElement {
    Field::new().label(label).child(Input::new(input))
}

fn switch_field(label: String, child: impl IntoElement) -> impl IntoElement {
    Field::new()
        .label(label)
        .bg(AppColors::Border.hsla())
        .p_2()
        .w_auto()
        .rounded_md()
        .child(child)
}

impl Render for CreateCustomerView {
    fn render(
        &mut self,
        window: &mut gpui_kit::Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex_1()
            .p_4()
            .child(
                div()
                    .child(if self.name.len() > 3 {
                        format!("{}", &self.name.as_str())
                    } else {
                        "Create a new Customer".to_string()
                    })
                    .text_2xl(),
            )
            .child(
                div()
                    .size_full()
                    .items_center()
                    .justify_center()
                    .child(field("Name".to_string(), &self.name_input))
                    .child(field("Email".to_string(), &self.email_input))
                    .child(field("Address".to_string(), &self.address_input))
                    .child(field("Instagram".to_string(), &self.instagram_url_input))
                    .child(field("Site url".to_string(), &self.site_url_input))
                    .child(field("Phone number".to_string(), &self.phone_number_input))
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .gap_2()
                            .mt_2()
                            .child(switch_field(
                                "Have been contaced?".to_string(),
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
                Button::new("create customer")
                    .primary()
                    .child("Save")
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.save_customer(cx);
                        window.push_notification(
                            format!(
                                "Customer {} created",
                                this.name.split_whitespace().next().unwrap_or("")
                            ),
                            cx,
                        );
                        cx.notify();
                    })),
            )
    }
}

impl CreateCustomerView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name = String::new();
        let email = String::new();
        let address = String::new();
        let instagram_url = String::new();
        let phone_number = String::new();
        let site_url = String::new();
        let contacted = false;
        let is_client = false;

        let name_input = cx.new(|cx| InputState::new(window, cx));
        let email_input = cx.new(|cx| InputState::new(window, cx));
        let address_input = cx.new(|cx| InputState::new(window, cx));
        let instagram_url_input = cx.new(|cx| InputState::new(window, cx));
        let site_url_input = cx.new(|cx| InputState::new(window, cx));
        let phone_number_input = cx.new(|cx| InputState::new(window, cx));

        cx.subscribe(&name_input, |this, input, _event: &InputEvent, cx| {
            this.name = input.read(cx).text().to_string();
        })
        .detach();

        cx.subscribe(&email_input, |this, input, _event: &InputEvent, cx| {
            this.email = input.read(cx).text().to_string();
        })
        .detach();

        cx.subscribe(&address_input, |this, input, _event: &InputEvent, cx| {
            this.address = input.read(cx).text().to_string();
        })
        .detach();

        cx.subscribe(
            &instagram_url_input,
            |this, input, _event: &InputEvent, cx| {
                this.instagram_url = input.read(cx).text().to_string();
            },
        )
        .detach();

        cx.subscribe(&site_url_input, |this, input, _event: &InputEvent, cx| {
            this.site_url = input.read(cx).text().to_string();
        })
        .detach();

        cx.subscribe(
            &phone_number_input,
            |this, input, _event: &InputEvent, cx| {
                this.phone_number = input.read(cx).text().to_string();
            },
        )
        .detach();

        let pool = cx.global::<DbPool>().0.clone();

        let repository = CustomerRepository::new(pool);

        Self {
            name,
            name_input,
            email_input,
            address,
            address_input,
            instagram_url_input,
            phone_number_input,
            site_url_input,
            email,
            contacted,
            instagram_url,
            phone_number,
            site_url,
            repository,
            is_client,
        }
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
        // 1. Reseta as strings internas da struct
        self.name.clear();
        self.email.clear();
        self.address.clear();
        self.instagram_url.clear();
        self.phone_number.clear();
        self.site_url.clear();
        self.is_client = false;
        self.contacted = false;

        self.name_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.email_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.address_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.instagram_url_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.site_url_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.phone_number_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        cx.notify();
    }

    pub fn save_customer(&mut self, cx: &mut Context<Self>) {
        let name = self.name.clone();
        let email = self.email.clone();
        let address = self.address.clone();
        let instagram_url = self.instagram_url.clone();
        let phone_number = self.phone_number.clone();
        let site_url = self.site_url.clone();

        let customer = Customer::<Draft> {
            id: Draft,
            name,
            email,
            phone_number,
            address: if address.is_empty() {
                None
            } else {
                Some(address)
            },
            instagram_url: if instagram_url.is_empty() {
                None
            } else {
                Some(instagram_url)
            },
            site_url: if site_url.is_empty() {
                None
            } else {
                Some(site_url)
            },
            is_client: self.is_client,
            contacted: self.contacted,
        };

        let repository = self.repository.clone();

        cx.spawn(async move |this, cx| {
            let result = repository.create_customer(customer).await;

            this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(_new_customer) => {
                        cx.emit(CreateCustomerEvent::Created);
                        this.reset_form(window, cx);
                    }
                    Err(e) => {
                        eprintln!("{}", e.to_string());
                    }
                };
                cx.notify();
            })
        })
        .detach();
    }
}
