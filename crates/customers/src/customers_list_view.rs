use gpui_kit::component::accordion::Accordion;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::tag::Tag;
use gpui_kit::component::{ActiveTheme, Icon, IconName, WindowExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Window,
};
use gpui_kit::{base::Disableable, div};
use shared::customer::{Customer, Persisted};
use shared::db::DbPool;
use shared::events::AppEvent;

use crate::customer_repository::CustomerRepository;

pub enum CustomerListViewEvent {
    OPEN(Customer),
}

impl EventEmitter<CustomerListViewEvent> for CustomerListView {}
impl EventEmitter<AppEvent> for CustomerListView {}

pub enum PatchField {
    CONTACTED,
    CLIENT,
}

#[derive(PartialEq)]
pub enum CustomerFilter {
    OnlyClient,
    Contacted,
    WithInstagram,
    WithSite,
    NoClient,
    NoContacted,
    NoInstagram,
    NoSite,
}

pub struct CustomerListView {
    customers: Vec<Customer<Persisted>>,
    filtered_customers: Vec<Customer<Persisted>>,

    // filters states
    filters: Vec<CustomerFilter>,
    filter_expanded: bool,

    // Query
    query: String,
    query_input: Entity<InputState>,

    // repository
    pub repository: CustomerRepository,

    pub open_customers: Vec<Customer<Persisted>>,
}

pub fn tag_item(label: &str, value: String, cx: &Context<CustomerListView>) -> impl IntoElement {
    div().flex().child(
        Tag::secondary().child(
            div()
                .flex()
                .gap_1()
                .child(
                    div()
                        .child(format!("{}", label))
                        .text_color(cx.theme().primary)
                        .font_family("Geist Mono")
                        .font_weight(FontWeight::BOLD),
                )
                .child(format!("{}", value)),
        ),
    )
}

pub fn customer_item(
    customer: Customer<Persisted>,
    is_selected: bool,
    cx: &Context<CustomerListView>,
) -> impl IntoElement {
    let id = customer.id.to_owned();
    div()
        .id(id.to_string())
        .hover(|f| f.border_1().border_color(cx.theme().selection))
        .h_auto()
        .w_full()
        .border_1()
        .border_color(if is_selected {
            cx.theme().primary
        } else {
            cx.theme().border
        })
        .when(!is_selected, |f| f.bg(cx.theme().accent))
        .when(is_selected, |f| f.bg(cx.theme().selection))
        .p_2()
        .child(
            div()
                .w_full()
                .min_w_full()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().child(format!("{}", customer.name)).text_2xl())
                .child(
                    div().flex().child(
                        Tag::primary()
                            .child(format!("{}", customer.email))
                            .w_auto()
                            .flex_shrink_1(),
                    ),
                )
                .w_auto()
                .child({
                    let metadata_tags = [
                        ("Instagram", customer.instagram_url.as_deref()),
                        ("Site", customer.site_url.as_deref()),
                        ("Address", customer.address.as_deref()),
                    ];

                    div().w_full().flex().mt_1().gap_2().children(
                        metadata_tags.into_iter().filter_map(|(label, val)| {
                            val.filter(|s| !s.trim().is_empty())
                                .map(|v| tag_item(label, v.to_string(), cx))
                        }),
                    )
                })
                .child(
                    div()
                        .flex()
                        .gap_5()
                        .mt_1()
                        .child(
                            div().flex().gap_2().items_center().child(
                                Switch::new(format!("{id}-contact-toggle"))
                                    .label("Our client?")
                                    .checked(customer.is_client)
                                    .on_change(cx.listener(move |this, value, window, cx| {
                                        this.set_boolean_field(
                                            customer.id,
                                            *value,
                                            PatchField::CLIENT,
                                            cx,
                                        );
                                        window.push_notification("Is client toggle", cx);
                                        cx.notify();
                                    })),
                            ),
                        )
                        .child(
                            div().flex().gap_2().items_center().child(
                                Switch::new(format!("{id}-client-toggle"))
                                    .label("Contacted?")
                                    .checked(customer.contacted.to_owned())
                                    .on_change(cx.listener(move |this, value, window, cx| {
                                        this.set_boolean_field(
                                            customer.id.to_owned(),
                                            *value,
                                            PatchField::CONTACTED,
                                            cx,
                                        );
                                        window.push_notification("Contacted toggle", cx);
                                        cx.notify();
                                    })),
                            ),
                        ),
                )
                .child(
                    div()
                        .w_full()
                        .flex()
                        .justify_end()
                        .gap_2()
                        .child(
                            Button::new(format!("go-to-details-{}", id.to_string()))
                                .primary()
                                .label("View")
                                .on_click(cx.listener(move |this, _, _window, cx| {
                                    cx.emit(CustomerListViewEvent::OPEN(customer.clone()));
                                    this.toggle_customer(customer.clone(), cx);
                                })),
                        )
                        .child(
                            Button::new(id.to_string())
                                .label("Delete")
                                .secondary()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.delete_customer(id.clone(), cx);
                                })),
                        ),
                ),
        )
}

impl Render for CustomerListView {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let filtered = self.filtered_customers.clone();
        let mut sorted = self.customers.clone();
        sorted.sort_by_key(|i| !filtered.contains(i));

        let customers_filtered = self.get_filtered(&sorted);

        div()
            .w_full()
            .h_full()
            .p_2()
            .gap_2()
            .max_h_full()
            .child(
                div()
                    .child(Input::new(&self.query_input).prefix(Icon::new(IconName::Search)))
                    .child(
                        Accordion::new("Filters")
                            .item(|item| {
                                item.title("Filters").open(self.filter_expanded).child(
                                    div()
                                        .child(
                                            Checkbox::new("only clients")
                                                .label("Only clients")
                                                .checked(
                                                    self.filters
                                                        .contains(&CustomerFilter::OnlyClient),
                                                )
                                                .on_change(cx.listener(|view, _this, _, cx| {
                                                    view.toggle_filter(CustomerFilter::OnlyClient);
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            Checkbox::new("Contacted")
                                                .label("Only contacted")
                                                .checked(
                                                    self.filters
                                                        .contains(&CustomerFilter::Contacted),
                                                )
                                                .on_change(cx.listener(|view, _this, _, cx| {
                                                    view.toggle_filter(CustomerFilter::Contacted);
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            Checkbox::new("With instagram")
                                                .label("Has instagram")
                                                .checked(
                                                    self.filters
                                                        .contains(&CustomerFilter::WithInstagram),
                                                )
                                                .on_change(cx.listener(|view, _this, _, cx| {
                                                    view.toggle_filter(
                                                        CustomerFilter::WithInstagram,
                                                    );
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            Checkbox::new("With site")
                                                .label("Has website")
                                                .checked(
                                                    self.filters
                                                        .contains(&CustomerFilter::WithSite),
                                                )
                                                .on_change(cx.listener(|view, _this, _, cx| {
                                                    view.toggle_filter(CustomerFilter::WithSite);
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            Checkbox::new("No client")
                                                .label("Not client")
                                                .checked(
                                                    self.filters
                                                        .contains(&CustomerFilter::NoClient),
                                                )
                                                .on_change(cx.listener(|view, _this, _, cx| {
                                                    view.toggle_filter(CustomerFilter::NoClient);
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            Checkbox::new("No contacted")
                                                .label("Not contacted")
                                                .checked(
                                                    self.filters
                                                        .contains(&CustomerFilter::NoContacted),
                                                )
                                                .on_change(cx.listener(|view, _this, _, cx| {
                                                    view.toggle_filter(CustomerFilter::NoContacted);
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            Checkbox::new("No instagram")
                                                .label("No instagram")
                                                .checked(
                                                    self.filters
                                                        .contains(&CustomerFilter::NoInstagram),
                                                )
                                                .on_change(cx.listener(|view, _this, _, cx| {
                                                    view.toggle_filter(CustomerFilter::NoInstagram);
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            Checkbox::new("No site")
                                                .label("No website")
                                                .checked(
                                                    self.filters.contains(&CustomerFilter::NoSite),
                                                )
                                                .on_change(cx.listener(|view, _this, _, cx| {
                                                    view.toggle_filter(CustomerFilter::NoSite);
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            div().flex().justify_end().w_full().mt_2().child(
                                                Button::new("clear-filters")
                                                    .ghost()
                                                    .disabled(self.filters.is_empty())
                                                    .label("Clear filters")
                                                    .on_click(cx.listener(|view, _, _, cx| {
                                                        view.clear_filters();
                                                        cx.notify();
                                                    })),
                                            ),
                                        ),
                                )
                            })
                            .on_toggle_click(cx.listener(|view, _this, _, cx| {
                                view.toggle_filter_accordion();
                                cx.stop_propagation();
                                cx.notify();
                            }))
                            .mt_2(),
                    ),
            )
            .child(
                div().mt_2().gap_2().flex().flex_col().children(
                    customers_filtered
                        .into_iter()
                        .map(|c| customer_item(c.to_owned(), filtered.contains(&c), cx)),
                ),
            )
    }
}

impl CustomerListView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let pool = App::global::<DbPool>(&cx).0.clone();
        let repository = CustomerRepository::new(pool);
        let query_input = cx.new(|cx| {
            InputState::new(window, cx)
                .clean_on_escape()
                .placeholder("Search customers")
        });

        cx.subscribe(&query_input, |this, input, _event: &InputEvent, cx| {
            let q = input
                .read(cx)
                .text()
                .to_string()
                .to_lowercase()
                .trim()
                .to_string();

            this.query = q.clone();

            if q.trim().len() > 0 {
                let c = this.customers.clone();
                let filtered_customers: Vec<Customer<Persisted>> = c
                    .into_iter()
                    .filter(|c| {
                        c.name.trim().to_lowercase().contains(&this.query.trim())
                            || c.email.trim().to_lowercase().contains(&this.query)
                            || c.site_url
                                .clone()
                                .is_some_and(|url| url.trim().to_lowercase().contains(&this.query))
                            || c.instagram_url
                                .clone()
                                .is_some_and(|url| url.trim().to_lowercase().contains(&this.query))
                            || c.address
                                .clone()
                                .is_some_and(|url| url.trim().to_lowercase().contains(&this.query))
                    })
                    .collect();

                this.filtered_customers = filtered_customers.clone()
            } else {
                this.filtered_customers = Vec::new();
            };
        })
        .detach();

        Self {
            customers: Vec::new(),
            filtered_customers: Vec::new(),
            repository,
            query: String::new(),
            open_customers: Vec::new(),
            query_input,
            filters: Vec::new(),
            filter_expanded: false,
        }
    }

    pub fn toggle_filter_accordion(&mut self) {
        self.filter_expanded = !self.filter_expanded
    }

    pub fn hydrate_customers(&mut self, cx: &mut Context<Self>) {
        let repo = self.repository.clone();
        cx.spawn(async move |this, cx| {
            let customers = repo.get_customers().await;
            match customers {
                Ok(data) => {
                    if let Err(error) = this.update(cx, |this, cx| {
                        this.customers = data.clone();
                        cx.emit(AppEvent::LoadedCustomers(data));
                        cx.notify();
                    }) {
                        eprintln!("{}", error.to_string())
                    }
                }
                Err(_) => {}
            }
        })
        .detach();
    }

    pub fn delete_customer(&mut self, customer_id: i64, cx: &mut Context<Self>) {
        let repo = self.repository.clone();
        cx.spawn(async move |this, cx| {
            if repo.delete_customer(customer_id).await.is_ok() {
                let _ = this.update(cx, |this, cx| {
                    this.hydrate_customers(cx);
                    cx.emit(AppEvent::DeletedCustomer(customer_id));
                });
            }
        })
        .detach();
        cx.notify();
    }

    pub fn set_boolean_field(
        &mut self,
        customer_id: i64,
        value: bool,
        field: PatchField,
        cx: &mut Context<Self>,
    ) {
        let repo = self.repository.clone();

        cx.spawn(async move |this, cx| {
            match field {
                PatchField::CONTACTED => {
                    repo.set_contacted(customer_id, value).await;
                }
                PatchField::CLIENT => {
                    repo.set_is_client(customer_id, value).await;
                }
            };
            this.update(cx, |this, cx| {
                this.hydrate_customers(cx);
                cx.notify();
            })
        })
        .detach();
    }

    pub fn toggle_customer(&mut self, customer: Customer, _cx: &mut Context<Self>) {
        if self.open_customers.contains(&customer) {
            let filtered: Vec<Customer> = self
                .customers
                .clone()
                .into_iter()
                .filter(|f| f.id != customer.id)
                .collect();

            self.open_customers = filtered;
            return;
        }
        self.open_customers.push(customer);
    }

    pub fn toggle_filter(&mut self, filter: CustomerFilter) {
        if self.filters.contains(&filter) {
            let pos = self.filters.iter().position(|f| *f == filter);

            if let Some(position_index) = pos {
                self.filters.remove(position_index);
                return;
            }
        }

        self.filters.push(filter);
    }

    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }

    fn this_customer_satifies_this_filter(
        &self,
        customer: &Customer,
        filter: &CustomerFilter,
    ) -> bool {
        match filter {
            CustomerFilter::Contacted => customer.contacted,
            CustomerFilter::OnlyClient => customer.is_client,
            CustomerFilter::WithInstagram => customer
                .instagram_url
                .as_ref()
                .map_or(false, |url| !url.trim().is_empty()),
            CustomerFilter::WithSite => customer
                .site_url
                .as_ref()
                .map_or(false, |url| !url.trim().is_empty()),
            CustomerFilter::NoClient => !customer.is_client,
            CustomerFilter::NoContacted => !customer.contacted,
            CustomerFilter::NoInstagram => {
                !self.this_customer_satifies_this_filter(customer, &CustomerFilter::WithInstagram)
            }
            CustomerFilter::NoSite => {
                !self.this_customer_satifies_this_filter(customer, &CustomerFilter::WithSite)
            }
        }
    }

    fn get_filtered(&self, customers: &Vec<Customer>) -> Vec<Customer> {
        if self.filters.is_empty() {
            return customers.clone();
        }

        customers
            .iter()
            .filter(|c| {
                self.filters
                    .iter()
                    .all(|f| self.this_customer_satifies_this_filter(c, f))
            })
            .cloned()
            .collect()
    }
}
