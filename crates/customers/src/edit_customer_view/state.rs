use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, Window,
    base::input::InputState,
    component::WindowExt,
};
use shared::{
    customer::{Customer, Persisted},
    db::DbPool,
    form_utils::text_input,
};

use crate::{
    customer_repository::{CustomerRepository, CustomerUpdateDraft},
    edit_customer_view::events::CustomerUpdateEvent,
};

pub struct CustomerUpdateView {
    pub customer: Option<Customer<Persisted>>,
    pub(super) name: Entity<InputState>,
    pub(super) email: Entity<InputState>,
    pub(super) phone_number: Entity<InputState>,
    pub(super) address: Entity<InputState>,
    pub(super) instagram_url: Entity<InputState>,
    pub(super) site_url: Entity<InputState>,
    pub(super) is_client: bool,
    pub(super) contacted: bool,
    pub(super) repository: CustomerRepository,
}

impl CustomerUpdateView {
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
        let repository = CustomerRepository::new(pool);

        let name = cx.new(|cx| InputState::new(window, cx));
        let email = cx.new(|cx| InputState::new(window, cx));
        let phone_number = cx.new(|cx| InputState::new(window, cx));
        let address = cx.new(|cx| InputState::new(window, cx));
        let instagram_url = cx.new(|cx| InputState::new(window, cx));
        let site_url = cx.new(|cx| InputState::new(window, cx));

        let mut view = Self {
            customer: None,
            name,
            email,
            phone_number,
            address,
            instagram_url,
            site_url,
            is_client: false,
            contacted: false,
            repository,
        };

        if let Some(c) = customer {
            view.set_customer(c, window, cx);
        }

        view
    }

    pub fn set_customer(
        &mut self,
        customer: Customer<Persisted>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.name.update(cx, |input, cx| {
            input.set_value(&customer.name, window, cx);
        });
        self.email.update(cx, |input, cx| {
            input.set_value(&customer.email, window, cx);
        });
        self.phone_number.update(cx, |input, cx| {
            input.set_value(&customer.phone_number, window, cx);
        });
        self.address.update(cx, |input, cx| {
            input.set_value(
                customer.address.as_deref().unwrap_or_default(),
                window,
                cx,
            );
        });
        self.instagram_url.update(cx, |input, cx| {
            input.set_value(
                customer.instagram_url.as_deref().unwrap_or_default(),
                window,
                cx,
            );
        });
        self.site_url.update(cx, |input, cx| {
            input.set_value(
                customer.site_url.as_deref().unwrap_or_default(),
                window,
                cx,
            );
        });

        self.is_client = customer.is_client;
        self.contacted = customer.contacted;
        self.customer = Some(customer);
    }

    pub fn set_is_client(&mut self, value: bool) {
        self.is_client = value;
        if value {
            self.contacted = true;
        }
    }

    pub fn set_contacted(&mut self, value: bool) {
        self.contacted = value;
    }

    pub fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(customer) = self.customer.clone() {
            self.set_customer(customer, window, cx);
            cx.notify();
        }
    }

    pub fn update_customer(&mut self, cx: &mut Context<Self>) {
        let Some(customer) = self.customer.clone() else {
            eprintln!("No customer set to update");
            return;
        };

        let draft = CustomerUpdateDraft {
            name: text_input(&self.name, cx),
            email: text_input(&self.email, cx),
            phone_number: text_input(&self.phone_number, cx),
            address: text_input(&self.address, cx),
            instagram_url: text_input(&self.instagram_url, cx),
            site_url: text_input(&self.site_url, cx),
            is_client: Some(self.is_client),
            contacted: Some(self.contacted),
        };

        let repository = self.repository.clone();
        let customer_id = customer.id;

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    repository.update_customer(customer_id, draft).await
                })
                .await;

            match result {
                Ok(updated_customer) => {
                    let _ = this.update_in(cx, |this, window, cx| {
                        window.push_notification(
                            format!("Customer {} updated", updated_customer.name),
                            cx,
                        );
                        this.set_customer(updated_customer.clone(), window, cx);
                        cx.emit(CustomerUpdateEvent::UpdatedCustomer(updated_customer));
                        cx.notify();
                    });
                }
                Err(e) => {
                    let _ = this.update_in(cx, |_this, window, cx| {
                        window.push_notification(format!("Error updating customer: {e}"), cx);
                    });
                    eprintln!("Error updating customer: {e}");
                }
            }
        })
        .detach();
    }
}

impl EventEmitter<CustomerUpdateEvent> for CustomerUpdateView {}
